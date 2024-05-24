use std::fmt::format;
use tonic::{Request, Response, Status};
use crate::proto::endpoint_server::Endpoint;
use crate::requests::{find_node, update_request};
use crate::signatures::*;
use crate::util::*;
use sha256::{digest};
use crate::endpoint::*;
use crate::proto::{AddTransactionRequest, AddTransactionResponse, Block, GenerateRequest, GenerateResponse, Node, NodeInfo, RetrieveBlockchainRequest, RetrieveBlockchainResponse, Signature, TransactionRequest, TransactionResponse};

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
pub async fn generate_block(blockchain:Vec<Block>, transactions:Vec<String>) -> Block{
    verify_blockchain(blockchain.clone());
    let most_recent=blockchain.get(blockchain.len()-1).unwrap();
    let prev_hash= hash_block(most_recent.clone());
    let mut nonce=0;
    loop {
        let cur=Block{prev_hash:prev_hash.clone(), nonce:nonce.clone(), merkle_root: hash_transactions(transactions.clone())};
        let prefix = create_prefix();
        if String::from_utf8(hash_block(cur.clone())).unwrap()[0..2]==prefix{return cur;};
        nonce+=1;
    }
}