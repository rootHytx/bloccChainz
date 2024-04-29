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

    async fn contact_nodes(&self, node:Node) -> Vec<NodeInfo>{
        let mut bootstraps = self.bootstraps.write().await;
        let mut res = bootstraps.clone().to_vec();
        for i in 0..bootstraps.len(){
            let cur_node = bootstraps.get(i).unwrap();
            let mut client = EndpointClient::connect(
                format!("http://{}:{}", cur_node.ip.to_string(), cur_node.port.to_string()))
                .await.expect("FAILURE CONNECTING TO BOOTSTRAP");
            let request = Request::new(UpdateRequest{ info: node.info.clone() });
            let response = client.update_node(request).await.unwrap();
            //println!("connection response: {:?}", response);
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
        println!("Got a request from {:?}", request.into_inner());
        let response = PingResponse{ message:"OK".to_string() };
        Ok(Response::new(response))
    }
    async fn join(&self, request: Request<JoinRequest>)  -> Result<Response<JoinResponse>, Status> {
        println!("Got a request from {}", request.get_ref().node.clone().unwrap().info.unwrap());
        let req = request.get_ref().clone().node.unwrap();
        let node = Node{ info:req.info, kbuckets:req.kbuckets, neighbours:req.neighbours };
        let neighbours = self.contact_nodes(node).await;
        println!("{:?}", neighbours);
        Ok(Response::new(JoinResponse{ neighbours }))
    }
}

async fn bootstrap_init (addr: SocketAddr) -> Result<(EndpointService, Node, SocketAddr), Box<dyn std::error::Error>>{
    let mut service = EndpointService::default();
    let node = service.setup(IP_ADDRESS.to_string(), true).await;
    let boot_addr = format!("{}:{}", node.info.clone().unwrap().ip, node.info.clone().unwrap().port).parse().unwrap();
    let url = format!("http://{}:{}", addr.ip().to_string(), addr.port().to_string());
    init_client(node.clone(), url).await.unwrap();
    Ok((service, node, boot_addr))
}

async fn bootstrap_server_init(service: EndpointService, node: Node, boot_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    Server::builder()
        .add_service(EndpointServer::new(service))
        .serve(boot_addr.clone()).await.expect("FAILURE INITIALIZING BOOTSTRAP SERVER");
    Ok(())
}

async fn create_server(addr:SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    let server_handle = task::spawn(async move{
        Server::builder()
            .add_service(NetworkServer::new(NetworkService::default()))
            .serve(addr.clone()).await.expect("FAILURE SERVING SERVER ADDRESS");
    });
    let boot_handle = task::spawn(async move {
        for _ in 0..N_BOOTSTRAPS {
            let (service, node, boot_addr) = bootstrap_init(addr).await.expect("FAILURE INITIALIZING BOOTSTRAP NODES");
            task::spawn(async move{
                bootstrap_server_init(service, node, boot_addr).await.expect("FAILURE SPAWNING BOOTSTRAP SERVER");
            });
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