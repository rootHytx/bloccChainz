//use client::init_client;
use crate::proto::network_client::NetworkClient;
use crate::proto::endpoint_client::EndpointClient;
use crate::proto::{PingRequest, PingResponse, JoinRequest, JoinResponse, UpdateRequest, UpdateResponse, GnRequest, GnResponse, NodeInfo, KBucket, Node};
use endpoint::*;
use util::*;
use node::*;
use proto::network_server::{Network, NetworkServer};
use proto::endpoint_server::{Endpoint, EndpointServer};
use rand::seq::IndexedRandom;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::task;
use tokio::task::JoinSet;
use tonic::{Request, Response, Status};
use tonic::transport::{Server};

mod endpoint;
mod util;
mod proto{
    tonic::include_proto!("kademlia");
}
mod node;
type BootstrapList = std::sync::Arc<tokio::sync::RwLock<Vec<NodeInfo>>>;
#[derive(Clone,Debug,Default)]
struct NetworkService{
    bootstraps: BootstrapList,
}

impl NetworkService{
    async fn print_bootstraps(&self){
        println!("All Bootstraps:");
        for i in 0..self.bootstraps.read().await.to_vec().len(){
            self.print_some_bootstrap(Option::from(i)).await;
        }
    }
    async fn print_some_bootstrap(&self, index:Option<usize>){
        match index {
            None => println!("First Bootstrap:\n{}", self.bootstraps.read().await.to_vec().get(0).unwrap()),
            Some(ind) => println!("\tBootstrap[{}]:\n\t{}\n", ind, self.bootstraps.read().await.to_vec().get(ind).unwrap()),
        }
    }
    async fn get_bootstraps(&self) -> Vec<NodeInfo>{
        self.bootstraps.read().await.to_vec()
    }

    async fn get_neighbours(&self, node:Node) -> Vec<NodeInfo>{
        let mut bootstraps = self.bootstraps.write().await;
        let mut res = bootstraps.clone().to_vec();
        for cur in bootstraps.clone().to_vec(){
            let response = update_node(cur.clone(), node.info.clone().unwrap()).await.expect("FAILURE CONTACTING BOOTSTRAP");
            for i in response.get_ref().neighbours.clone(){
                if !res.contains(&&i){
                    res.push(i);
                }
            }
        }
        if node.info.clone().unwrap().bootstrap {bootstraps.push(node.info.unwrap());};
        res
    }

    async fn clone(&self) -> NetworkService{
        NetworkService{
            bootstraps: self.bootstraps.clone(),
        }
    }
}

#[tonic::async_trait]
impl Network for NetworkService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request from {:?}", request.get_ref().clone());
        let node = request.get_ref().node.clone().unwrap();
        let url = format!("http://{}:{}", node.ip, node.port);
        if EndpointClient::connect(url).await.is_err(){
            return Ok(Response::new(PingResponse{ response:false }));
        }
        Ok(Response::new(PingResponse{ response:true }))
    }
    async fn join(&self, request: Request<JoinRequest>)  -> Result<Response<JoinResponse>, Status> {
        let req = request.get_ref().clone().node.unwrap();
        println!("Got a request from {}@{}", req.info.clone().unwrap().id, req.info.clone().unwrap().port);
        let neighbours = self.get_neighbours(req.clone()).await;
        Ok(Response::new(JoinResponse{ neighbours }))
    }
}

async fn create_server(addr:SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    let server_addr = addr.clone();
    let server_handle = task::spawn(async move{
        Server::builder()
            .add_service(NetworkServer::new(NetworkService::default()))
            .serve(server_addr).await.expect("FAILURE SERVING SERVER ADDRESS");
    });
    let boot_handle = task::spawn(async move {
        for _ in 0..N_BOOTSTRAPS {
            create_client(true, addr).await.expect("FAILURE INITIALIZING CLIENT");
        }
    });
    tokio::try_join!(server_handle, boot_handle).expect("FAILURE JOINING TASKS FOR SERVER CREATION");
    Ok(())
}

#[tokio::main]
async fn main(){
    let addr = create_address();
    println!("Server Address: {:?}", addr);
    create_server(addr).await.expect("FAILED TO CREATE SERVER");
}