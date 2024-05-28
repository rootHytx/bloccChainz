use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::os::fd::{AsFd, AsRawFd};
use rand::RngCore;
use sha256::digest;
use tokio::net::TcpSocket;

use crate::util::*;
use crate::proto::{NodeInfo, BucketNode, KBucket, Node};
mod proto {
    tonic::include_proto!("kademlia");
}

impl NodeInfo{

    pub fn clone(&self) -> NodeInfo{
        NodeInfo{ id: self.id.clone(), ip: self.ip.clone(), port: self.port.clone(), pkey:self.pkey.clone(), bootstrap: self.bootstrap.clone(), miner:self.miner.clone() }
    }
}
impl Display for NodeInfo{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tNodeInfo:\n\t\tID: {}\n\t\tIP: {}\n\t\tPORT: {}\n\t\tBOOTSTRAP: {}", self.id, self.ip, self.port, self.bootstrap)
    }
}

impl KBucket{
    pub fn new() -> KBucket{
        let nodes = Vec::new();
        KBucket{ nodes }
    }
    pub fn insert(&mut self, new: NodeInfo, distance: i64){
        self.nodes.push(BucketNode{position:(distance - 2_i64.pow(distance.ilog2() as u32)) as u32, info:Option::from(new.clone())})
    }
    pub fn get_node(&self, distance:i64) -> Option<NodeInfo>{
        for i in self.nodes.clone(){
            if i.position == (distance - 2_i64.pow(distance.ilog2() as u32)) as u32{
                return i.info.clone();
            }
        }
        None
    }
    pub fn contains(&self, distance: i64) -> bool{
        if self.get_node(distance)!=None{return true;};
        false
    }
    pub fn print(&self) -> String{
        let mut res = String::new();
        for i in 0..self.nodes.len(){
            if self.nodes.get(i) != Some(&BucketNode { position: 0, info: None }){
                res = format!("{}\t\t\t{:?}@{}\n", res, self.nodes.get(i).unwrap().info.clone().unwrap(), i.clone());
            }
        };
        res
    }
    pub fn remove(&mut self, _node:String, distance:i64){
        for i in 0..self.nodes.clone().len(){
            if let Some(cur) = self.nodes.get(i).clone(){
                if cur.position== (distance - 2_i64.pow(distance.ilog2() as u32)) as u32 {
                    self.nodes.remove(i);
                }
            }
        }
    }
    pub fn get_quantity(&self) -> i64{
        let mut res=0;
        for i in self.nodes.to_vec(){
            if i.info!=None{
                res+=1;
            }
        }
        res
    }
}

impl Node{
    pub fn init_routes() -> Vec<KBucket>{
        let mut routes = Vec::new();
        for _i in 0..N_BUCKETS{routes.push(KBucket::new());};
        routes
    }
    pub fn distance(one:String, two:String) -> i64{
        i64::from_str_radix(one.as_str(), 16).unwrap() ^ i64::from_str_radix(two.as_str(), 16).unwrap()
    }
    pub fn new(ip: String, port:Option<u32>, miner:bool) -> Self{
        let mut input = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut input);
        let input = digest(&input);
        let destination;
        let bootstrap;
        if port.clone()!=None{ destination=format!("{}:{}", ip, port.unwrap());bootstrap=true }
        else{ destination=format!("{}:0", ip);bootstrap=false };
        let socket = bind(destination).unwrap().expect("FAILURE BINDING SOCKET");
        let (skey, pkey) = new_key();
        let info = NodeInfo{ id: input[..ID_SIZE].to_string(), ip, port: socket.local_addr().unwrap().port() as u32, pkey, bootstrap, miner};
        let routes = Self::init_routes();
        let node = Node{ info:Option::from(info.clone()), skey, kbuckets:routes, neighbours:Vec::new(), blockchain:Vec::new()};
        node
    }
    pub fn clone(&self) -> Node{
        Node{ info: Option::from(self.info.clone()), skey:self.skey.clone(),
            kbuckets: self.kbuckets.clone(), neighbours:self.neighbours.clone(), blockchain:self.blockchain.clone()}
    }

    pub async fn new_route(&mut self, new: NodeInfo) -> bool{
        let distance = Self::distance(self.info.clone().unwrap().id, new.clone().id);
        let quantity = self.get_quantity();
        if !self.kbuckets.get(distance.ilog2() as usize).unwrap().contains(distance) && (self.info.clone().unwrap().bootstrap || quantity.get(distance.ilog2() as usize).unwrap()<&(K_SIZE as i64)){
            self.kbuckets.get_mut(distance.ilog2() as usize).unwrap().insert(new.clone(), distance);
            self.neighbours.push(new.clone().id);
            return true;
        };
        false
    }
    pub fn get_neighbour(&self, other:String) -> Option<NodeInfo>{
        let distance = Self::distance(self.info.clone().unwrap().id, other);
        if distance==0{return None}
        else{
            if let Some(cur) = self.kbuckets.get(distance.ilog2() as usize).unwrap().clone().get_node(distance){
                return Option::from(cur.clone())
            }
        }
        return None
    }
    pub fn get_neighbours(&self) -> Vec<NodeInfo>{
        let mut res = Vec::new();
        for i in 0..self.neighbours.len(){
            if let Some(cur) = self.get_neighbour(self.neighbours.get(i).unwrap().clone()){
                res.push(cur)
            }
        }
        res
    }
    pub fn get_bucket(&self, node:String) -> Vec<NodeInfo>{
        let distance = Self::distance(self.info.clone().unwrap().id, node);
        if distance==0{return Vec::new()}
        let mut res= Vec::new();
        for i in self.kbuckets.get(distance.ilog2() as usize).unwrap().clone().nodes{
            res.push(i.info.unwrap());
        };
        while res.len()==0{
            for i in self.kbuckets.get((distance.ilog2()- 1) as usize).unwrap().clone().nodes{
                res.push(i.info.unwrap());
            };
            for i in self.kbuckets.get((distance.ilog2()+ 1) as usize).unwrap().clone().nodes{
                res.push(i.info.unwrap());
            };
        };
        res
    }
    pub fn get_closest_nodes(&self, node:NodeInfo) -> Vec<NodeInfo>{
        if self.info.clone().unwrap().bootstrap{
            let all_nodes = self.neighbours.clone();
            let mut res = Vec::new();
            let mut temp = Vec::new();
            for _i in 0..self.kbuckets.len(){
                temp.push(0i64);
            }
            for i in all_nodes{
                let distance = Self::distance(node.clone().id, i.clone());
                if distance > 0{
                    if temp.get(distance.ilog2() as usize).unwrap()< &(K_SIZE as i64) {
                        if let Some(cur) = self.get_neighbour(i.clone()){
                            res.push(cur);
                            temp.insert(distance.ilog2() as usize, temp.get(distance.ilog2() as usize).unwrap() + 1i64);
                        }
                    }
                }
            }
            return res
        }
        Vec::new()
    }
    pub fn remove(&mut self, node:String){
        let distance = Self::distance(self.info.clone().unwrap().id, node.clone());
        if let Some(bucket) = self.kbuckets.get_mut(distance.ilog2() as usize) {
            bucket.remove(node.clone(), distance)
        }
    }
    pub fn get_quantity(&self) ->Vec<i64>{
        let mut res = Vec::new();
        for i in self.kbuckets.clone(){
            res.push(i.get_quantity());
        }
        res
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res = format!("Node:\n\t{}\n\tKBUCKETS:\n", self.info.clone().unwrap());
        for i in 0..self.kbuckets.len(){
            res = format!("{}\t\tINDEX: {}\n{}", res, i, self.kbuckets.get(i).unwrap().print());
        }
        write!(f, "{}", res)
    }
}