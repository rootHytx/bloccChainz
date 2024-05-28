use endpoint::*;
use util::*;
use rand::Rng;
use requests::*;
use std::error::Error;
use std::fmt::format;
use std::io;
use core::net::SocketAddr;
use std::time::Duration;
use tokio::task;
use tokio::signal;
use tonic::{Response, Status};
use tonic::transport::{Endpoint, Server};
use node::*;
use proto::miner_server::*;
use crate::proto::{NodeInfo, BucketNode, KBucket, Node, JoinResponse};
use crate::proto::endpoint_server::EndpointServer;
use crate::signatures::*;
use tokio_util::sync::CancellationToken;
use blockchain::*;
use nodes_init::*;
mod endpoint;
mod util;
mod node;
mod proto {
    tonic::include_proto!("kademlia");
}
mod blockchain;
mod requests;
mod signatures;
mod nodes_init;
fn entry(n:i32) -> String{
    loop{
        let mut number = String::new();
        if n==0{println!("Create how many miner nodes? ")}
        else{println!("Create how many client nodes? ")}
        io::stdin()
            .read_line(&mut number)
            .expect("Failed to read line");
        match number.trim().parse::<i32>() {
            Ok(_) => return number,
            Err(ref e) => println!("ERROR PARSING INTEGER {}", e),
        }
    };
}

#[tokio::main]
async fn main(){
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    let mut miner_number = entry(0);
    let mut client_number = entry(1);
    tokio::select! {
        _ = cloned_token.cancelled() => {
            println!("CLOSING");
        }
        _ = tokio::time::sleep(Duration::from_secs(0)) => {tokio::spawn(async move{
            let mut res= Vec::new();
            for i in 0..miner_number.trim().parse::<i32>().unwrap(){
                res.push(create_client(None, true).await.expect("FAILURE INITIALIZING CLIENT"));
            };
            for i in 0..client_number.trim().parse::<i32>().unwrap(){
                res.push(create_client(None, false).await.expect("FAILURE INITIALIZING CLIENT"));
            };
        });}
    }
    let test = tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {token.cancel();},
            Err(err) => { eprintln!("Unable to listen for shutdown signal: {}", err); },
        }
    });
    tokio::try_join!(test).expect("FAILURE INITIALIZING CLIENTS");
}