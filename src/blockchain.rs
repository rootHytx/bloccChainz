use crate::proto::Bid;
use std::fmt::format;
use std::net::SocketAddr;
use tonic::{Request, Response, Status};
use crate::proto::endpoint_server::Endpoint;
use crate::requests::{find_node, update_request};
use crate::signatures::*;
use crate::util::*;
use sha256::{digest};
use crate::endpoint::*;
use crate::proto::{AbortRequest, AbortResponse, Block, MineRequest, MineResponse, Node, NodeInfo, RetrieveBlockchainRequest, RetrieveBlockchainResponse, Signature, TransactionRequest, TransactionResponse};
use crate::proto::miner_server::Miner;

#[derive(Clone,Debug,Default)]
pub struct MinerService{ source:std::sync::Arc<tokio::sync::RwLock<String>>, }
impl MinerService{
    pub async fn generate_block(&self, prev: Option<Block>, transactions:Vec<String>) -> Block{
        //verify_blockchain(blockchain.clone());
        //let most_recent=blockchain.get(blockchain.len()-1).unwrap();
        let prev_hash= hash_block(prev.clone().expect("REASON"));
        let mut nonce=0;
        loop {
            let cur=Block{prev_hash:prev_hash.clone(), nonce:nonce.clone(), merkle_root: hash_transactions(transactions.clone())};
            let prefix = create_prefix();
            if String::from_utf8(hash_block(cur.clone())).unwrap()[0..2]==prefix{return cur;};
            nonce+=1;
        }
    }
    pub async fn init_source(&mut self, source_id:String){
        self.source = std::sync::Arc::from(tokio::sync::RwLock::from(source_id));
    }
}
#[tonic::async_trait]
impl Miner for MinerService{
    async fn mine(&self, request: Request<MineRequest>) -> Result<Response<MineResponse>, Status> {
        let new = self.generate_block(request.get_ref().clone().previous, request.get_ref().clone().transactions).await;

        Ok(Response::new(MineResponse{source_id:request.get_ref().clone().source_id, new:Option::from(new)}))
    }

    async fn abort(&self, request: Request<AbortRequest>) -> Result<Response<AbortResponse>, Status> {
        todo!()
    }
}

#[derive(Clone,Debug,Default)]
pub struct MinerInfo{
    pub current_transactions: std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
    pub active_bids: std::sync::Arc<tokio::sync::RwLock<Vec<Bid>>>,
    pub queued: bool,
    pub queued_transactions:std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
    pub miner_ip:String,
    pub miner_port:String,
}
impl MinerInfo {
    pub fn new() -> MinerInfo{
        let mut current_transactions = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        let mut queued_transactions = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        let mut active_bids = std::sync::Arc::from(tokio::sync::RwLock::from(Vec::new()));
        MinerInfo{current_transactions, active_bids, queued:false, queued_transactions, miner_ip:String::new(), miner_port:String::new() }
    }
    pub async fn write_transaction(&mut self, transaction:String) -> String{
        let mut transaction_list = self.current_transactions.write().await.clone();
        return if transaction_list.len() < TRANSACTION_NUMBER as usize {
            transaction_list.push(transaction);
            "processed".to_string()
        } else {
            self.queued_transactions.write().await.clone().push(transaction);
            self.queued = true;
            "queued".to_string()
        }
    }
    pub fn reserve_address(&mut self, ip:String){
        self.miner_ip = ip.clone();
        let addr = format!("{}:0", ip);
        let addr = bind(addr).unwrap().expect("FAILURE BINDING SOCKET");
        self.miner_port = addr.local_addr().unwrap().clone().port().to_string();
    }
}

pub fn hash_block(b:Block) -> Vec<u8>{
    let v=format!("{}{}{}", String::from_utf8(b.clone().prev_hash).unwrap(), b.clone().nonce, String::from_utf8(b.clone().merkle_root).unwrap());
    digest(v).as_bytes().to_vec()
}

pub fn create_hashes(vals:Vec<String>) -> Vec<String>{
    let mut res= Vec::new();
    for i in 1..vals.len(){
        if i<vals.len() && i%2==0{ res.push(format!("{}{}", digest(vals.get(i-1).unwrap()), digest(vals.get(i-1).unwrap()))) }
        else{res.push(digest(vals.get(i).unwrap()))}
    }
    res
}
pub fn hash_transactions(transactions:Vec<String>) -> Vec<u8>{
    let mut to_hash=transactions.clone();
    while to_hash.len()>1{
      to_hash=create_hashes(to_hash.clone());
    };
    to_hash.get(0).unwrap().as_bytes().to_vec()
}
pub fn verify_blockchain(blockchain:Vec<Block>){
    for i in 1..blockchain.len(){
        let cur_b_header_hash = blockchain.get(i).unwrap().clone().prev_hash;
        let prev_b = blockchain.get(i-1).unwrap();
        let hash_prev=hash_block(prev_b.clone());
        assert!(cur_b_header_hash.eq(&hash_prev));
    }
}

pub fn create_prefix() -> String{
    let res=String::new();
    for i in 0..PREFIX_LENGTH{
        format!("{}{}", res, 0);
    };
    res
}

