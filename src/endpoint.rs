use tonic::{Request, Response, Status};
use crate::proto::endpoint_server::Endpoint;
use crate::proto::{Node, NodeInfo, FindNodeRequest, FindNodeResponse, UpdateRequest, UpdateResponse, JoinRequest, JoinResponse, NeighboursRequest, NeighboursResponse, RemoveRequest, RemoveResponse, Signature, TransactionRequest, TransactionResponse, Block, RetrieveBlockchainRequest, RetrieveBlockchainResponse, GenerateRequest, GenerateResponse, AddTransactionRequest, AddTransactionResponse};
use crate::requests::{find_node, update_request};
use crate::signatures::*;
use crate::util::*;
use crate::blockchain::*;

pub type SafeNode = std::sync::Arc<tokio::sync::RwLock<Node>>;

#[derive(Clone,Debug,Default)]
pub struct EndpointService{
    node:SafeNode,
    bootstraps:std::sync::Arc<tokio::sync::RwLock<Vec<NodeInfo>>>,
    blocks:std::sync::Arc<tokio::sync::RwLock<Vec<Block>>>,
}

impl EndpointService{
    pub fn is_correct_key(&self, local:Vec<u8>, sent:Vec<u8>){ assert!(local.eq(&sent)) }
    pub async fn setup_client(&mut self, ip: String, port:Option<u32>, miner:bool) -> Node{
        let node = Node::new(ip, port, miner);
        self.node = SafeNode::from(tokio::sync::RwLock::from(node.clone()));
        self.bootstraps = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
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
            println!("Got a request from {}@{}", sender.id, sender.port);
            if let Some(info) = own.info.clone() {
                let bootstraps = self.bootstraps.read().await.clone();
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
                    update_request(own.clone(), req.clone(), format_url(i.ip.clone(), i.port.clone().to_string())).await;
                }
                update_request(own.clone(), req.clone(), format_url(own.info.clone().unwrap().ip, own.info.clone().unwrap().port.to_string())).await;
            }
        };
        let hash = sign_join_response(neighbours.clone(), self.node.read().await.clone().blockchain, self.node.read().await.clone().skey);
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        Ok(Response::new(JoinResponse{ neighbours, blockchain:self.node.read().await.clone().blockchain, sign:Option::from(sign) }))
    }
    async fn find_node(&self, request: Request<FindNodeRequest>) -> Result<Response<FindNodeResponse>, Status>{
        verify_find_node_request(request.get_ref().clone());
        println!("NODE {} FINDING {}", self.node.read().await.info.clone().unwrap().id, request.get_ref().clone().target);
        if let Some(res) = self.node.read().await.get_neighbour(request.get_ref().clone().target){
            let hash = sign_find_node_response(res.clone(),self.node.read().await.clone().skey);
            let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
            return Ok(Response::new(FindNodeResponse{node:Option::from(res), sign:Option::from(sign)}))
        }
        let closest = self.node.read().await.get_bucket(request.get_ref().clone().target);
        for i in closest{
            let response = find_node(self.node.read().await.clone(), request.get_ref().clone().target, format_url(i.ip, i.port.to_string()))
                .await;
            if let Some(res) = response{
                let hash = sign_find_node_response(res.clone(), self.node.read().await.skey.clone());
                let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
                return Ok(Response::new(FindNodeResponse{node:Option::from(res), sign:Option::from(sign)}))
            }
        }
        Ok(Response::new(FindNodeResponse{node:None, sign:None}))
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
        let hash = sign_update_response(true, self.node.read().await.skey.clone());
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        return Ok(Response::new(UpdateResponse{ response:true, sign:Option::from(sign) }))
    }
    async fn get_neighbours(&self, request: Request<NeighboursRequest>) -> Result<Response<NeighboursResponse>, Status>{
        verify_neighbours_request(request.get_ref().clone());
        let neighbours=self.node.read().await.get_neighbours();
        let hash = sign_neighbours_response(neighbours.clone(), self.node.read().await.skey.clone());
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        return Ok(Response::new(NeighboursResponse{ neighbours, sign:Option::from(sign)}));
    }
    async fn remove_node(&self, request: Request<RemoveRequest>) -> Result<Response<RemoveResponse>, Status>{
        verify_remove_request(request.get_ref().clone());
        let mut node = self.node.write().await;
        if node.get_neighbour(request.get_ref().node.clone().unwrap().id)!=None{
            node.remove(request.get_ref().node.clone().unwrap().id);
        }
        let hash = sign_remove_response(true,node.skey.clone());
        let sign = Signature{hash, pkey:self.node.read().await.info.clone().unwrap().pkey.clone()};
        Ok(Response::new(RemoveResponse{ success:true, sign:Option::from(sign)}))
    }

    async fn transaction(&self, request: Request<TransactionRequest>) -> Result<Response<TransactionResponse>, Status> {
        //MinerClient::connect()
        todo!()
    }

    async fn retrieve_blockchain(&self, request: Request<RetrieveBlockchainRequest>) -> Result<Response<RetrieveBlockchainResponse>, Status> {
        todo!()
    }
    async fn generate(&self, request: Request<GenerateRequest>) -> Result<Response<GenerateResponse>, Status> {
        verify_generate_request(request.get_ref().clone());
        let blockchain=request.get_ref().clone().blockchain;
        let transactions = request.get_ref().clone().transactions;
        let new = generate_block(blockchain, transactions).await;
        let info=self.node.read().await.clone().info.unwrap();
        let hash=sign_generate_response(info.id.clone(), new.clone(), info.pkey.clone());
        let sign = Option::from(Signature{hash, pkey:info.pkey.clone()});
        Ok(Response::new(GenerateResponse{source_id:info.id.clone(), new:Option::from(new.clone()), sign}))
    }

    async fn add_transaction(&self, request: Request<AddTransactionRequest>) -> Result<Response<AddTransactionResponse>, Status> {
        todo!()
    }
}