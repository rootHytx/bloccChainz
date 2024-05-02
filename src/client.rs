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

async fn client(mut node:Node, url:String) -> Result<(), Box<dyn Error>>{
    println!("NODE: {}@{}", node.clone().info.unwrap().id, node.info.clone().unwrap().port);
    let response = init_client(node.clone(), url).await.unwrap();
    for i in response.get_ref().neighbours.clone(){
        update_node(node.info.clone().unwrap(), i).await.expect("cona");
    }
    Ok(())
}

#[tokio::main]
async fn main(){
    let main_task = task::spawn(async move{
        let addr = format_address();
        for _i in 0..20{
            create_client(false, addr).await.expect("FAILURE INITIALIZING CLIENT");
        }
    });
    let other_task = task::spawn(async move{
        //println!("AAAAAAAAAAAAAAAAAAAAAAAAAA");
        //let mut query = String::new();
        loop {
           /*io::stdin()
               .read_line(&mut query)
               .expect("Failed to read line");
           println!("{}", query);*/
        }
    });
    tokio::try_join!(main_task, other_task);
}