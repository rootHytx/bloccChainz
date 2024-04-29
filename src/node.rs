use std::fmt;
use std::fmt::{Display, Formatter, write};
use std::net::UdpSocket;
use rand::RngCore;
use sha256::digest;

use crate::util::*;
use crate::proto::{NodeInfo, BucketNode, KBucket, Node};
mod proto {
    tonic::include_proto!("kademlia");
}

impl NodeInfo{
    pub fn clone(&self) -> NodeInfo{
        NodeInfo{ id: self.id.clone(), ip: self.ip.clone(), port: self.port.clone(), bootstrap: self.bootstrap.clone() }
    }
    pub fn distance(&self, other: NodeInfo) -> i32{
        i32::from_str_radix(self.id.as_str(), 16).unwrap() ^ i32::from_str_radix(other.id.as_str(), 16).unwrap()
    }
}
impl Display for NodeInfo{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tNodeInfo:\n\t\tID: {}\n\t\tIP: {}\n\t\tPORT: {}\n\t\tBOOTSTRAP: {}", self.id, self.ip, self.port, self.bootstrap)
    }
}

impl BucketNode{
    pub fn new() -> BucketNode{
        BucketNode{ info: None }
    }
}

impl KBucket{
    pub fn new(values:i32) -> KBucket{
        let mut nodes = Vec::new();
        for _ in 0..values{
            nodes.push(BucketNode::new())
        }
        KBucket{ nodes }
    }
    pub fn insert(&mut self, new: NodeInfo, distance: i32){
        //println!("NODE POS: {}", distance - 2_i32.pow(distance.ilog2() as u32));
        self.nodes.insert((distance - 2_i32.pow(distance.ilog2() as u32)) as usize, BucketNode{ info:Option::from(new)});
        //println!("{}", self.print());
    }
    pub fn contains(&self, distance: i32) -> bool{
        let contains = self.nodes.get((distance - 2_i32.pow(distance.ilog2() as u32)) as usize);
        if contains==Some(&BucketNode { info: None }){
            return false;
        };
        true
    }
    pub fn print(&self) -> String{
        let mut res = String::new();
        //println!("CURRENT BUCKET LENGTH: {}", self.nodes.len());
        for i in 0..self.nodes.len(){
            if self.nodes.get(i) != Some(&BucketNode { info: None }){
                res = format!("{}\t\t\t{:?}\n", res, self.nodes.get(i).unwrap().info.clone().unwrap());
            }
        };
        res
    }
    pub fn get_node(&self, distance:i32) -> Option<NodeInfo>{
        if let Some(cur ) = self.nodes.get((distance - 2_i32.pow(distance.ilog2() as u32)) as usize){
            return Option::from(cur.info.clone().unwrap())
        }
        return None
    }
}

impl Node{
    pub fn init_routes() -> Vec<KBucket>{
        let mut routes = Vec::new();
        for i in 0..N_BUCKETS{
            let values = 2_i32.pow(i.try_into().unwrap());
            let temp = KBucket::new(values);
            routes.push(temp);
        };
        routes
    }
    pub fn new(ip: String, bootstrap: bool) -> Self{
        let mut input = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut input);
        let input = digest(&input);
        let destination = format!("{}:0", ip);
        let socket = UdpSocket::bind(destination).expect("couldn't bind to address");
        let info = NodeInfo{ id: input[..ID_SIZE].to_string(), ip, port: socket.local_addr().unwrap().port() as u32, bootstrap};
        let mut routes = Self::init_routes();
        let mut node = Node{ info:Option::from(info.clone()), kbuckets:routes, neighbours:Vec::new()};
        node
    }
    pub fn clone(&self) -> Node{
        Node{ info: Option::from(self.info.clone()), kbuckets: self.kbuckets.clone(), neighbours:self.neighbours.clone()}
    }

    pub fn new_route(&mut self, new: NodeInfo){
        let distance = self.info.clone().unwrap().distance(new.clone());
        if let Some(bucket) = self.kbuckets.get_mut(distance.ilog2() as usize) {
            if !bucket.contains(distance){
                bucket.insert(new.clone(), distance);
                self.neighbours.push(new.clone().id);
            }
        }
    }
    pub fn get_neighbour(&self, other:String) -> Option<NodeInfo>{
        let distance = i32::from_str_radix(self.info.clone().unwrap().id.as_str(), 16).unwrap() ^ i32::from_str_radix(other.as_str(), 16).unwrap();
        println!("DISTANCE: {}", distance);
        if let Some(cur) = self.kbuckets.get(distance.ilog2() as usize).unwrap().clone().get_node(distance){
            return Option::from(cur.clone())
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
    pub fn get_closest_nodes(&self, node:NodeInfo) -> Vec<NodeInfo>{
        if self.info.clone().unwrap().bootstrap{
            let all_nodes = self.neighbours.clone();
            let mut res = Vec::new();
            let mut temp = Vec::new();
            for _i in 0..self.kbuckets.len(){
                temp.push(0i32);
            }
            for i in all_nodes{
                let distance = node.clone().distance(NodeInfo{ id:i.clone(), ip:"".to_string(), port: 0, bootstrap: false });
                if distance > 0{
                    if temp.get(distance.ilog2() as usize).unwrap()< &(K_SIZE as i32) {
                        if let Some(cur) = self.get_neighbour(i.clone()){
                            res.push(cur);
                            temp.insert(distance.ilog2() as usize, temp.get(distance.ilog2() as usize).unwrap() + 1i32);
                        }
                    }
                }
            }
            return res
        }
        Vec::new()
    }
}


impl fmt::Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut res = format!("Node:\n\t{}\n\tKBUCKETS:\n", self.info.clone().unwrap());
        for i in 0..self.kbuckets.len(){
            res = format!("{}\t\tINDEX: {}\n{}", res, i, self.kbuckets.get(i).unwrap().print());
        }
        //println!("ROUTING TABLE LENGTH: {}", self.kbuckets.len());
        write!(f, "{}", res)
    }
}

