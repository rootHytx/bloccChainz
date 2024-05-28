use std::fmt::format;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public, Id};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use sha256::{digest, TrySha256Digest};
use crate::blockchain::hash_block;
use crate::proto::{Block, FindNodeRequest, FindNodeResponse, JoinRequest, JoinResponse, MineRequest, MineResponse, NeighboursRequest, NeighboursResponse, Node, NodeInfo, ObtainTransactionsRequest, ObtainTransactionsResponse, RemoveRequest, RemoveResponse, RetrieveBlockchainRequest, RetrieveBlockchainResponse, TransactionRequest, TransactionResponse, UpdateBlockchainRequest, UpdateBlockchainResponse, UpdateRequest, UpdateResponse};

fn sign(content:String, skey: Vec<u8>) -> Vec<u8>{
    let skey = Rsa::private_key_from_pem(&*skey).unwrap();
    let skey = PKey::from_rsa(skey).unwrap();
    let mut signer = Signer::new(MessageDigest::sha256(), &skey).unwrap();
    signer.update(content.as_bytes()).unwrap();
    let signature = signer.sign_to_vec().unwrap();
    signature
}

pub fn verify(content:String, signature:Vec<u8>, pkey: Vec<u8>){
    let pkey = PKey::public_key_from_pem(&*pkey).unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
    verifier.update(content.as_bytes()).unwrap();
    assert!(verifier.verify(&signature).unwrap())
}
pub fn sign_join_request(node:Node, skey:Vec<u8>) -> Vec<u8>{
    sign(node.to_string(), skey)
}
pub fn verify_join_request(request:JoinRequest){
    let content = request.node.unwrap();
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_join_response(neighbours:Vec<NodeInfo>, blockchain:Vec<Vec<u8>>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = String::new();
    for i in neighbours{
        input = format!("{}{}", input, i.to_string());
    }
    for i in blockchain{
        input = format!("{}{}", input, String::from_utf8(i).unwrap());
    }
    sign(input, skey)
}
pub fn verify_join_response(response:JoinResponse){
    let neighbours = response.neighbours;
    let blockchain = response.blockchain;
    let mut content = String::new();
    for i in neighbours{
        content = format!("{}{}", content, i.to_string());
    }
    for i in blockchain{
        content = format!("{}{}", content, String::from_utf8(i).unwrap());
    }
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_find_node_request(source_id:String, node:String, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id.clone(), node);
    sign(input, skey)
}
pub fn verify_find_node_request(request:FindNodeRequest){
    let content = format!("{}{}", request.source_id, request.target);
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_find_node_response(source_id:String, info:NodeInfo, skey:Vec<u8>) -> Vec<u8>{
    let content = format!("{}{}", source_id, info.to_string());
    sign(content, skey)
}
pub fn verify_find_node_response(response:FindNodeResponse){
    let content = format!("{}{}", response.source_id, response.node.unwrap());
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_remove_request(source_id:String, node:NodeInfo, skey:Vec<u8>) -> Vec<u8>{
    //let input = format!("{}{}", source_id, node.to_string());
    let input = format!("{}", node);
    sign(input, skey)
}
pub fn verify_remove_request(request:RemoveRequest){
    //let content = format!("{}{}", request.source_id, request.node.unwrap().to_string());
    let content = format!("{}", request.node.unwrap());
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_remove_response(source_id:String, b:bool, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id, b.to_string());
    sign(input, skey)
}
pub fn verify_remove_response(response:RemoveResponse){
    let content = format!("{}{}", response.source_id, response.success);
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_update_request(source_id:String, nodes:Vec<NodeInfo>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = source_id.clone();
    for i in nodes{
        input = format!("{}{}", input, i.to_string())
    }
    sign(input, skey)
}
pub fn verify_update_request(request:UpdateRequest){
    let neighbours = request.neighbours;
    let mut content = request.source_id.clone();
    for i in neighbours{
        content = format!("{}{}", content, i.to_string())
    }
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_update_response(source_id:String, b:bool, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id, b.to_string());
    sign(input, skey)
}
pub fn verify_update_response(response:UpdateResponse){
    let content = format!("{}{}", response.source_id, response.response);
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_neighbours_request(source_id:String, skey:Vec<u8>) -> Vec<u8>{
    sign(source_id, skey)
}
pub fn verify_neighbours_request(request:NeighboursRequest){
    let content = request.source_id;
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_neighbours_response(source_id:String, neighbours:Vec<NodeInfo>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = source_id.clone();
    for i in neighbours{
        input = format!("{}{}", input, i.to_string());
    }
    sign(input, skey)
}
pub fn verify_neighbours_response(response:NeighboursResponse){
    let neighbours = response.neighbours;
    let mut content = response.source_id.clone();
    for i in neighbours{
        content=format!("{}{}", content, i.to_string());
    }
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_transaction_request(source_id:String, value:i32, destination:String, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}{}", source_id, value, destination);
    sign(input, skey)
}
pub fn verify_transaction_request(request:TransactionRequest){
    let source = request.source_id;
    let value = request.value;
    let destination = request.destination;
    let content = format!("{}{}{}", source, value, destination);
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().clone().pkey;
    verify(content, signature, pkey)
}
pub fn sign_transaction_response(source_id:String, state:String, skey:Vec<u8>) -> Vec<u8>{
    let content = format!("{}{}", source_id, state);
    sign(content, skey)
}
pub fn verify_transaction_response(response:TransactionResponse){
    let input = format!("{}{}", response.source_id, response.state);
    let signature = response.sign.clone().unwrap().hash;
    let pkey = response.sign.clone().unwrap().clone().pkey;
    verify(input, signature, pkey)
}
pub fn sign_obtain_transactions_request(source_id:String, skey:Vec<u8>) -> Vec<u8>{
    let input = source_id.clone();
    sign(input, skey)
}
pub fn verify_obtain_transactions_request(request:ObtainTransactionsRequest){
    let content = request.source_id.clone();
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().clone().pkey;
    verify(content, signature, pkey)
}
pub fn sign_obtain_transactions_response(source_id:String, transactions:Vec<String>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = source_id.clone();
    for i in transactions.clone(){
        input = format!("{}{}", input, i.clone());
    };
    sign(input,skey)
}
pub fn verify_obtain_transactions_response(response:ObtainTransactionsResponse){
    let mut content = response.source_id.clone();
    for i in response.transactions.clone(){
        content = format!("{}{}", content, i.clone());
    };
    let signature = response.sign.clone().unwrap().hash;
    let pkey = response.sign.clone().unwrap().clone().pkey;
    verify(content, signature, pkey)
}
pub fn sign_retrieve_blockchain_request(source_id:String, skey:Vec<u8>) -> Vec<u8>{
    sign(source_id.clone(), skey.clone())
}
pub fn verify_retrieve_blockchain_request(request: RetrieveBlockchainRequest){
    let id = request.source_id;
    let signature = request.sign.clone().unwrap().hash;
    let pkey= request.sign.clone().unwrap().pkey;
    verify(id, signature, pkey)
}
pub fn sign_retrieve_blockchain_response(source_id:String, blockchain:Vec<Block>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = source_id.clone();
    for i in blockchain{
      input = format!("{}{}", input, String::from_utf8(hash_block(i.clone())).unwrap())
    };
    sign(input, skey)
}
pub fn verify_retrieve_blockchain_response(response: RetrieveBlockchainResponse){
    let blockchain = response.blockchain;
    let mut content = response.source_id.clone();
    for i in blockchain{
      content = format!("{}{}", content, String::from_utf8(hash_block(i.clone())).unwrap())
    };
    let signature = response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_update_blockchain_request(source_id:String, new:Block, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id, String::from_utf8(hash_block(new.clone())).unwrap());
    sign(input, skey)
}
pub fn verify_update_blockchain_request(request:UpdateBlockchainRequest){
    let source_id = request.source_id;
    let new = request.new.unwrap();
    let content = format!("{}{}", source_id, String::from_utf8(hash_block(new.clone())).unwrap());
    let signature = request.sign.clone().unwrap().hash;
    let pkey= request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_update_blockchain_response(source_id:String, skey:Vec<u8>) -> Vec<u8>{
    sign(source_id.clone(), skey.clone())
}
pub fn verify_update_blockchain_response(response:UpdateBlockchainResponse){
    let source_id = response.source_id;
    let signature = response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(source_id, signature, pkey)
}

/*pub fn sign_mine_request(source_id:String, prev:Block, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id, String::from_utf8(hash_block(prev)).unwrap());
    sign(input, skey)
}
pub fn verify_mine_request(request:MineRequest){
    let content = format!("{}{}", request.source_id, String::from_utf8(hash_block(request.previous.unwrap())).unwrap());
    let signature = request.sign.clone().unwrap().hash;
    let pkey= request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_mine_response(source_id:String, new:Block, skey:Vec<u8>) -> Vec<u8>{
    let input = format!("{}{}", source_id, String::from_utf8(hash_block(new)).unwrap());
    sign(input, skey.clone())
}
pub fn verify_mine_response(response:MineResponse){
    let content = format!("{}{}", response.source_id, String::from_utf8(hash_block(response.new.unwrap())).unwrap());
    let signature = response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}*/