use std::fmt::format;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use tokio::task;
use tonic::{Response, Status};
use tonic::codegen::http::Request;
use tonic::transport::Server;
use crate::endpoint::EndpointService;
use crate::proto;
use crate::proto::{JoinResponse, Node, UpdateResponse, UpdateRequest, NodeInfo, GnResponse, GnRequest};
use crate::proto::endpoint_client::EndpointClient;
use crate::proto::endpoint_server::EndpointServer;
use crate::proto::network_client::NetworkClient;
pub const ID_SIZE: usize = 4; //Size of the NODE_ID (truncate hash output, see node.rs)
pub const N_BUCKETS: usize = ID_SIZE * 4; //For each bit of the NODE_ID, add one bucket (if ID_SIZE is in bytes, bytes*8 = bits)
pub const K_SIZE: usize = 5; //How many nodes per bucket
pub const N_BOOTSTRAPS: i32 = 3; //Number of bootstrap nodes
pub const IP_ADDRESS: &str = "127.0.0.1";

pub fn create_address() -> SocketAddr {
    let destination = format!("{}:0", IP_ADDRESS).to_string();
    let socket = UdpSocket::bind(destination).expect("FAILURE BINDING ADDRESS");
    socket.local_addr().unwrap()
}

pub fn format_address() -> SocketAddr{
    let mut port = String::new();
    println!("Input the port for the desired network:");
    io::stdin()
        .read_line(&mut port)
        .expect("Failed to read line");
    return match port.trim().parse::<i32>() {
        Ok(parsed_int) => format!("{}:{}", IP_ADDRESS, parsed_int).parse().unwrap(),
        Err(ref e) => {
            println!("Failed to read port: {:?}", e);
            format_address()
        },
    }
}

pub async fn init_client(mut node: Node, url: String) ->  Result<Response<JoinResponse>, Status>{
    let mut client = NetworkClient::connect(url).await.unwrap();
    let request = tonic::Request::new(proto::JoinRequest{ node: Some(node.clone())});
    let response = client.join(request).await?;
    Ok(response)
}

pub async fn update_node(receiver:NodeInfo, node:NodeInfo) -> Result<Response<UpdateResponse>, Status>{
    let url = format!("http://{}:{}", receiver.ip, receiver.port);
    println!("CONTACTING {}@{} TO INTRODUCE NEIGHBOUR {}@{}", receiver.id.clone(), receiver.port.clone(), node.id.clone(), node.port.clone());
    let mut client = EndpointClient::connect(url).await.expect("FAILURE CONNECTING TO CLIENT SERVER");
    let request = tonic::Request::new(proto::UpdateRequest{ info: Option::from(node.clone()) });
    let response = client.update_node(request).await?;
    Ok(response)
}

pub async fn serve_client(service: EndpointService, node: Node, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    Server::builder()
        .add_service(EndpointServer::new(service))
        .serve(addr.clone()).await.expect("FAILURE INITIALIZING BOOTSTRAP SERVER");
    Ok(())
}

pub async fn create_client(boot:bool, addr:SocketAddr) -> Result<(), Box<dyn std::error::Error>>{
    let mut service = EndpointService::default();
    let node = service.setup(IP_ADDRESS.to_string(), boot).await;
    let server_node = node.clone();
    task::spawn(async move{
        let addr:SocketAddr = format!("{}:{}", server_node.info.clone().unwrap().ip, server_node.info.clone().unwrap().port).parse().unwrap();
        serve_client(service, server_node.clone(), addr).await.expect("FAILURE SPAWNING BOOTSTRAP SERVER");
    });
    let url = format!("http://{}:{}", addr.ip().clone().to_string(), addr.port().clone().to_string());
    let neighbours = init_client(node.clone(), url).await.unwrap().get_ref().neighbours.clone();
    for i in neighbours{
        update_node(node.info.clone().unwrap(), i).await.expect("FAILURE CONTACTING ORIGINAL NODE");
    };
    Ok(())
}


pub async fn retrieve_neighbours(url:String) -> Result<Response<GnResponse>, Status>{
    let mut client = EndpointClient::connect(url).await.expect("FAILURE CONNECTING TO CLIENT URL");
    let request = tonic::Request::new(GnRequest{});
    let response = client.get_neighbours(request).await?;
    Ok(response)
}