use std::fmt::format;
use std::net::{SocketAddr, UdpSocket};
use std::io;
use rand::seq::IndexedRandom;
use tonic::{Request, Response, Status};
use tonic::transport::Server;
use proto::kademlia_server::{Kademlia, KademliaServer};
use crate::proto::{PingRequest, PingResponse, ConnectRequest, ConnectResponse};
use constants::*;
use node::*;
use routing_table::*;
use crate::proto::kademlia_client::KademliaClient;

mod constants;
mod node;
mod proto{
    tonic::include_proto!("kademlia");
}
mod routing_table;
type BootstrapList = std::sync::Arc<tokio::sync::RwLock<Vec<Node>>>;
#[derive(Debug, Default)]
struct KademliaService{
    bootstraps: BootstrapList,
}

impl KademliaService{
    async fn add_bootstrap(&self, new:Node) {
        self.bootstraps.write().await.push(new);
    }
    async fn init(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
        for _i in 0..N_BOOTSTRAPS{
            let boot=Node::new(IP_ADDRESS.to_string());
            self.add_bootstrap(boot.clone()).await;
            let mut client = KademliaClient::connect(
                format!("http://{}:{}", addr.ip().to_string(), addr.port().to_string())
            ).await?;
            let request = tonic::Request::new(proto::ConnectRequest{
                    node_id : boot.info.id.clone(),
                    ip : boot.info.ip.clone(),
                    port: u32::from(boot.info.port.clone()),
                    bootstrap:true,
                }
            );
            let response = client.connect_network(request).await?;
            println!("Response: {:?}", response.get_ref().nodes);
        }
        Ok(())
    }
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
        let response = ConnectResponse{ nodes };
        Ok(Response::new(response))
    }
}

fn create_address() -> SocketAddr {
    let destination = format!("{}:0", IP_ADDRESS).to_string();
    let socket = UdpSocket::bind(destination).expect("couldn't bind to address");
    socket.local_addr().unwrap()
}

async fn create_server(addr:SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    let server_init: tokio::task::JoinHandle<Result<_, Box<dyn std::error::Error + Send + Sync>>>= tokio::spawn(async move {
        let service = KademliaService::default();
        let server=Server::builder()
            .add_service(KademliaServer::new(service))
            .serve(addr.clone())
            .await?;
        Ok(())
    });
    let bootstraps_init = tokio::spawn(async move {
        let service = KademliaService::default();
        service.init(addr).await.expect("COULDN'T INITIALIZE BOOTSTRAP NODES");
        service.print_bootstraps().await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });
    let result = tokio::try_join!(server_init, bootstraps_init)?;
    println!("{:?}", result.1.unwrap());
    Ok(())
}

#[tokio::main]
async fn main(){
    let addr = create_address();
    println!("Server Address: {:?}", addr);
    create_server(addr).await.expect("FAILED TO CREATE SERVER");
}