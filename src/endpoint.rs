use prost::Message;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use crate::proto::{Node, NodeInfo, FindNodeRequest, FindNodeResponse, UpdateRequest, UpdateResponse, JoinRequest, JoinResponse, NeighboursRequest, NeighboursResponse, RemoveRequest, RemoveResponse, Signature, TransactionRequest, TransactionResponse, Block, RetrieveBlockchainRequest, RetrieveBlockchainResponse, UpdateBlockchainRequest, UpdateBlockchainResponse, ObtainTransactionsRequest, ObtainTransactionsResponse, CreateBidRequest, CreateBidResponse, BidValueRequest, BidValueResponse};
use crate::proto::miner_server::*;
use crate::requests::{find_node, mine_request, neighbours_request, transaction_request, update_blockchain_request, update_request};
use crate::signatures::*;
use crate::util::*;
use crate::blockchain::*;
use crate::proto::endpoint_server::Endpoint;

pub type SafeNode = std::sync::Arc<tokio::sync::RwLock<Node>>;
#[derive(Clone,Debug,Default)]
pub struct EndpointService{
    node:SafeNode,
    bootstraps:std::sync::Arc<tokio::sync::RwLock<Vec<NodeInfo>>>,
    blocks:std::sync::Arc<tokio::sync::RwLock<Vec<Block>>>,
    miner_info: std::sync::Arc<tokio::sync::RwLock<MinerInfo>>,
    transaction_list:std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
}

impl EndpointService{
    pub async fn is_correct_key(&self, source_id:String, sign:Signature){
        if let Some(neighbour) = self.node.read().await.get_neighbour(source_id){
            assert!(neighbour.pkey.eq(&sign.pkey))
        }
    }
    pub async fn setup_client(&mut self, ip: String, port:Option<u32>, miner:bool) -> Node{
        let node = Node::new(ip, port, miner);
        self.node = SafeNode::from(tokio::sync::RwLock::from(node.clone()));
        self.bootstraps = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        self.blocks = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        self.transaction_list = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        if miner{
            self.miner_info = std::sync::Arc::from(tokio::sync::RwLock::from(MinerInfo::new()));
            self.miner_info.write().await.reserve_address(self.node.read().await.clone().info.unwrap().ip);
            let miner = self.miner_info.read().await.clone();
            let addr = format_addr(miner.miner_ip, miner.miner_port);
            let source_id = node.clone().info.unwrap().id;
            tokio::spawn(async move {
                let mut service = MinerService::default();
                service.init_source(source_id).await;
                Server::builder()
                    .add_service(MinerServer::new(service))
                    .serve(addr.clone()).await.expect("FAILURE SETTING UP MINER SERVICE");
            });
        };
        node
    }
}
#[tonic::async_trait]
impl Endpoint for EndpointService{
    async fn join(&self, request: Request<JoinRequest>)  -> Result<Response<JoinResponse>, Status> {
        let req = request.get_ref().clone().node.unwrap().info;
        verify_join_request(request.get_ref().clone());
        let mut neighbours = Vec::new();
        if let Some(sender) = req {
            let own = self.node.read().await.clone();
            let bootstraps = self.bootstraps.read().await.clone();
            println!("Got a request from {}@{}", sender.id, sender.port);
            if let Some(info) = own.info.clone() {
                if !info.bootstrap {
                    println!("NOT A BOOTSTRAP DESTINATION, PLEASE CHECK THE AVAILABLE NODES");
                    let hash=sign_join_response(Vec::new(), Vec::new(), own.skey.clone());
                    let sign = Signature{hash, pkey:own.info.clone().unwrap().pkey.clone()};
                    return Ok(Response::new(JoinResponse { neighbours: Vec::new(), blockchain:Vec::new(), sign:Option::from(sign) }));
                }
                if sender.bootstrap { neighbours = own.get_neighbours(); } else { neighbours = own.get_closest_nodes(sender.clone()); };
                neighbours.push(info.clone());
                let mut req = Vec::new();
                req.push(sender.clone());
                for i in bootstraps.clone() {
                    if i != sender && !neighbours.contains(&i.clone()) {
                        neighbours.push(i.clone());
                    }
                }
            }
        };
        let hash = sign_join_response(neighbours.clone(), self.node.read().await.clone().blockchain, self.node.read().await.clone().skey);
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        Ok(Response::new(JoinResponse{ neighbours, blockchain:self.node.read().await.clone().blockchain, sign:Option::from(sign) }))
    }
    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status>{
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_find_node_request(request.get_ref().clone());
        println!("NODE {} FINDING {}", self.node.read().await.info.clone().unwrap().id, request.get_ref().clone().target);
        if let Some(res) = self.node.read().await.get_neighbour(request.get_ref().clone().target){
            let hash = sign_find_node_response(self.node.read().await.clone().info.unwrap().id, res.clone(),self.node.read().await.clone().skey);
            let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
            return Ok(Response::new(FindNodeResponse{source_id:self.node.read().await.clone().info.unwrap().id,node:Option::from(res), sign:Option::from(sign)}))
        }
        let closest = self.node.read().await.get_bucket(request.get_ref().clone().target);
        for i in closest{
            let response = find_node(self.node.read().await.clone(), request.get_ref().clone().target, format_url(i.ip, i.port.to_string()))
                .await;
            if let Some(res) = response{
                let hash = sign_find_node_response(self.node.read().await.clone().info.unwrap().id,res.clone(), self.node.read().await.skey.clone());
                let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
                return Ok(Response::new(FindNodeResponse{source_id:self.node.read().await.clone().info.unwrap().id,node:Option::from(res), sign:Option::from(sign)}))
            }
        }
        Ok(Response::new(FindNodeResponse{source_id:self.node.read().await.clone().info.unwrap().id,node:None, sign:None}))
    }
    async fn update_node(&self, request: Request<UpdateRequest>) -> Result<Response<UpdateResponse>, Status> {
        verify_update_request(request.get_ref().clone());
        let nodes = request.get_ref().clone().neighbours;
        for i in nodes{
            self.node.write().await.new_route(i.clone()).await;
            if i.bootstrap{
                self.bootstraps.write().await.push(i.clone());
            }
        }
        let hash = sign_update_response(self.node.read().await.clone().info.unwrap().id,true, self.node.read().await.skey.clone());
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        return Ok(Response::new(UpdateResponse{ source_id:self.node.read().await.clone().info.unwrap().id,response:true, sign:Option::from(sign) }))
    }
    async fn get_neighbours(&self, request: Request<NeighboursRequest>) -> Result<Response<NeighboursResponse>, Status>{
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_neighbours_request(request.get_ref().clone());
        let neighbours=self.node.read().await.get_neighbours();
        let hash = sign_neighbours_response(self.node.read().await.clone().info.unwrap().id,neighbours.clone(), self.node.read().await.skey.clone());
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        return Ok(Response::new(NeighboursResponse{ source_id:self.node.read().await.clone().info.unwrap().id,neighbours, sign:Option::from(sign)}));
    }
    async fn remove_node(&self, request: Request<RemoveRequest>) -> Result<Response<RemoveResponse>, Status>{
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_remove_request(request.get_ref().clone());
        let mut node = self.node.write().await;
        if node.get_neighbour(request.get_ref().node.clone().unwrap().id)!=None{
            node.remove(request.get_ref().node.clone().unwrap().id);
        }
        let hash = sign_remove_response(node.clone().info.unwrap().id,true,node.skey.clone());
        let sign = Signature{hash, pkey:node.clone().info.clone().unwrap().pkey.clone()};
        Ok(Response::new(RemoveResponse{ source_id:node.clone().info.unwrap().id,success:true, sign:Option::from(sign)}))
    }

    async fn transaction(&self, request: Request<TransactionRequest>) -> Result<Response<TransactionResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_transaction_request(request.get_ref().clone());
        let mut status = "".to_string();
        let node = self.node.read().await.clone();
        let blocks = self.blocks.read().await.clone();
        let mut miner = self.miner_info.write().await.clone();
        let transaction = format!("{}->{}->{}", request.get_ref().sender, request.get_ref().value, request.get_ref().destination);
        if node.info.clone().unwrap().miner && !miner.current_transactions.read().await.clone().contains(&transaction){
            println!("Node {}: {} RECEIVED TRANSACTION FROM {}, VALUE: {}", node.info.clone().unwrap().id, request.get_ref().destination, request.get_ref().sender, request.get_ref().value);
            status = miner.write_transaction(transaction.clone()).await;
            if status=="queued"{
                let new = mine_request(node.clone().info.unwrap().id, blocks.get(blocks.len()-1).unwrap().clone(),
                             miner.current_transactions.read().await.clone(), format_url(miner.miner_ip, miner.miner_port)).await;
                let boots = self.bootstraps.read().await.clone();
                let boot = boots.get(0).unwrap();
                update_blockchain_request(node.clone(), new.clone().unwrap(), format_url(boot.clone().ip, boot.clone().port.to_string())).await;
            };
        };
        if node.info.clone().unwrap().bootstrap{
            let n = neighbours_request(node.clone(), format_url(node.info.clone().unwrap().ip, node.info.clone().unwrap().port.to_string())).await;
            for i in n{
                if self.bootstraps.read().await.clone().contains(&i.clone()){continue;};
                let url = format_url(i.ip, i.port.to_string());
                transaction_request(node.clone(), request.get_ref().clone().sender, request.get_ref().clone().value as i32, request.get_ref().clone().destination, url).await;
            }
        }
        if request.get_ref().clone().destination==node.info.clone().unwrap().id{let mut t = self.transaction_list.write().await.clone(); t.push(transaction);};
        let info=node.clone().info.unwrap();
        let hash=sign_transaction_response(info.clone().id,status.clone(), node.skey.clone());
        let sign = Option::from(Signature{hash, pkey:info.clone().pkey});
        Ok(Response::new(TransactionResponse{source_id:info.clone().id,state:status, sign}))
    }

    async fn obtain_transactions(&self, request: Request<ObtainTransactionsRequest>) -> Result<Response<ObtainTransactionsResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_obtain_transactions_request(request.get_ref().clone());
        let node = self.node.read().await.clone();
        let info=node.info.unwrap();
        let hash=sign_obtain_transactions_response(info.clone().id, self.transaction_list.clone().read().await.clone(), node.skey.clone());
        let sign = Option::from(Signature{hash, pkey:info.clone().pkey});
        Ok(Response::new(ObtainTransactionsResponse{source_id:info.clone().id,transactions:self.transaction_list.clone().read().await.clone(), sign}))
    }

    async fn retrieve_blockchain(&self, request: Request<RetrieveBlockchainRequest>) -> Result<Response<RetrieveBlockchainResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_retrieve_blockchain_request(request.get_ref().clone());
        let blockchain = self.blocks.read().await.clone();
        let info=self.node.read().await.clone().info.unwrap();
        let hash=sign_retrieve_blockchain_request(info.id.clone(), self.node.read().await.clone().skey);
        let sign = Option::from(Signature{hash, pkey:info.pkey.clone()});
        Ok(Response::new(RetrieveBlockchainResponse{source_id:self.node.read().await.clone().info.unwrap().id,blockchain, sign}))
    }

    async fn update_blockchain(&self, request: Request<UpdateBlockchainRequest>) -> Result<Response<UpdateBlockchainResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        verify_update_blockchain_request(request.get_ref().clone());
        let mut temp = self.blocks.read().await.clone();
        temp.push(request.get_ref().clone().new.unwrap());
        verify_blockchain(temp);
        todo!()
    }

    async fn create_bid(&self, request: Request<CreateBidRequest>) -> Result<Response<CreateBidResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        todo!()
    }

    async fn bid_value(&self, request: Request<BidValueRequest>) -> Result<Response<BidValueResponse>, Status> {
        self.is_correct_key(request.get_ref().clone().source_id, request.get_ref().clone().sign.unwrap()).await;
        todo!()
    }
}