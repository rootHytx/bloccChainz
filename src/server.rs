use std::net::{SocketAddr, UdpSocket};
use std::io;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use proto::kademlia_server::{Kademlia, KademliaServer};
use crate::proto::{PingRequest, PingResponse, ConnectRequest, ConnectResponse};
type BootstrapList = std::sync::Arc<tokio::sync::RwLock<Vec<Node>>>;

mod node;
mod constants;
mod proto{
    tonic::include_proto!("kademlia");
}
use node::Node;
use constants::*;
#[derive(Debug, Default)]
struct KademliaService{
    bootstraps: BootstrapList,
}

impl KademliaService{
    async fn add_bootstrap(&self, new:Node) {
        self.bootstraps.write().await.push(new);
    }
    async fn init(&self) {
        for _i in 0..N_BOOTSTRAPS{
            self.add_bootstrap(Node::new("127.0.0.1".to_string())).await;
        }
    }
    async fn print_bootstraps(&self){
        println!("Current Bootstraps:\n{:?}", self.bootstraps.read().await)
    }
}

#[tonic::async_trait]
impl Kademlia for KademliaService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request from {:?}", request.into_inner());
        let response = PingResponse{ message:"OK".to_string() };
        Ok(Response::new(response))
    }
    async fn connect_network(&self, request: Request<ConnectRequest>)  -> Result<Response<ConnectResponse>, Status> {
        println!("Got a request from {:?}", request.into_inner());
        let mut nodes = Vec::new();

        nodes.push("cona".to_string());
        nodes.push("conaça".to_string());
        let response = ConnectResponse{ nodes };
        Ok(Response::new(response))
    }
}

fn create_address() -> SocketAddr {
    let destination = "127.0.0.1:0";
    let socket = UdpSocket::bind(destination).expect("couldn't bind to address");
    socket.local_addr().unwrap()
}

async fn create_server(addr:SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    let server = KademliaService::default();
    server.init().await;
    server.print_bootstraps().await;
    Server::builder()
        .add_service(KademliaServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main(){
    let addr = create_address();
    println!("Server Address: {:?}", addr);
    create_server(addr).await.expect("FAILED TO CREATE SERVER");
}