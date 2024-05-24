use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private, Public, Id};
use openssl::rsa::Rsa;
use openssl::sign::{Signer, Verifier};
use sha256::{digest, TrySha256Digest};
use crate::blockchain::hash_block;
use crate::proto::{Block, FindNodeRequest, FindNodeResponse, GenerateRequest, JoinRequest, JoinResponse, NeighboursRequest, NeighboursResponse, Node, NodeInfo, RemoveRequest, RemoveResponse, UpdateRequest, UpdateResponse};

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
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_find_node_request(node:String, skey:Vec<u8>) -> Vec<u8>{
    sign(node, skey)
}
pub fn verify_find_node_request(request:FindNodeRequest){
    let content = request.target;
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_find_node_response(info:NodeInfo, skey:Vec<u8>) -> Vec<u8>{
    sign(info.to_string(), skey)
}
pub fn verify_find_node_response(response:FindNodeResponse){
    let content = response.node.unwrap();
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_remove_request(node:NodeInfo, skey:Vec<u8>) -> Vec<u8>{
    sign(node.to_string(), skey)
}
pub fn verify_remove_request(request:RemoveRequest){
    let content = request.node.unwrap();
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_remove_response(b:bool, skey:Vec<u8>) -> Vec<u8>{
    sign(b.to_string(), skey)
}
pub fn verify_remove_response(response:RemoveResponse){
    let content = response.success;
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content.to_string(), signature, pkey)
}
pub fn sign_update_request(nodes:Vec<NodeInfo>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = String::new();
    for i in nodes{
        input = format!("{}{}", input, i.to_string())
    }
    sign(input, skey)
}
pub fn verify_update_request(request:UpdateRequest){
    let neighbours = request.neighbours;
    let mut content = String::new();
    for i in neighbours{
        content = format!("{}{}", content, i.to_string())
    }
    let signature = request.sign.clone().unwrap().hash;
    let pkey = request.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_update_response(b:bool, skey:Vec<u8>) -> Vec<u8>{
    sign(b.to_string(), skey)
}
pub fn verify_update_response(response:UpdateResponse){
    let content = response.response;
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
pub fn sign_neighbours_response(neighbours:Vec<NodeInfo>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = String::new();
    for i in neighbours{
        input = format!("{}{}", input, i.to_string());
    }
    sign(input, skey)
}
pub fn verify_neighbours_response(response:NeighboursResponse){
    let neighbours = response.neighbours;
    let mut content = String::new();
    for i in neighbours{
        content=format!("{}{}", content, i.to_string());
    }
    let signature= response.sign.clone().unwrap().hash;
    let pkey= response.sign.clone().unwrap().pkey;
    verify(content, signature, pkey)
}
pub fn sign_generate_request(id:String, blockchain:Vec<Block>, transactions:Vec<String>, skey:Vec<u8>) -> Vec<u8>{
    let mut input = id.clone();
    for i in blockchain{
      input=format!("{}{}", input, String::from_utf8(hash_block(i)).unwrap())
    };
    for i in transactions{
      input=format!("{}{}", input, i);
    };
    sign(input, skey)
}
pub fn verify_generate_request(req:GenerateRequest){

}
pub fn sign_generate_response(id:String, new:Block, skey:Vec<u8>) -> Vec<u8>{
    let mut input = id.clone();
    input=format!("{}{}", input, String::from_utf8(hash_block(new.clone())).unwrap());
    sign(input, skey)
}
pub fn verify_generate_response(id:String, new:Block, pkey:Vec<u8>){

}