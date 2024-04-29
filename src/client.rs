use endpoint::*;
use util::*;
use std::error::Error;
use std::fmt::format;
use std::io;
use core::net::SocketAddr;
use tokio::task;
use tonic::{Response, Status};
use tonic::transport::{Endpoint, Server};
use proto::network_client::NetworkClient;
use node::*;
use crate::proto::{NodeInfo, BucketNode, KBucket, Node, JoinResponse};
use crate::proto::endpoint_server::EndpointServer;

mod endpoint;
mod util;
mod node;
mod proto {
    tonic::include_proto!("kademlia");
}

fn create_url() -> String{
    let url = format!("http://{}", IP_ADDRESS).to_string();
    let mut port = String::new();
    println!("Input the port for the desired network:");
    io::stdin()
        .read_line(&mut port)
        .expect("Failed to read line");
    return match port.trim().parse::<i32>() {
        Ok(parsed_int) => format!("{}:{}", url, parsed_int),
        Err(ref e) => {
            println!("Failed to read port: {:?}", e);
            String::from("")
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let mut service = EndpointService::default();
    let mut node = service.setup(IP_ADDRESS.to_string(), false).await;
    let addr: SocketAddr = format!("{}:{}", node.info.clone().unwrap().ip, node.info.clone().unwrap().port).parse().expect("UNABLE TO PARSE ADDRESS");
    task::spawn(async move {
        Server::builder().add_service(EndpointServer::new(EndpointService::default()))
            .serve(addr).await.expect("FAILED TO CREATE NODE SERVER");
    });
    println!("NODE INFO: {}", node.clone());
    let mut url = "".to_string();
    while url=="".to_string(){
        url = create_url();
    }
    println!("{}",url);
    let response = init_client(node.clone(), url).await.unwrap();
    println!("Response: {:?}", response.get_ref().neighbours);
    node.new_route(Node::new(IP_ADDRESS.to_string(), false).info.unwrap());
    println!("{:?}", node.get_neighbours());
    Ok(())
}