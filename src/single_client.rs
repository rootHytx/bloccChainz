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

fn operations()->Vec<String>{
    let mut operations = Vec::new();
    operations.push("Show Neighbours".to_string());
    operations.push("Make Transaction".to_string());
    operations.push("List My Transactions".to_string());
    operations.push("Start New Bid".to_string());
    operations.push("Participate on Existing Bid".to_string());
    operations.push("List Participating Bids".to_string());
    operations.push("List All Bids".to_string());
    operations
}
async fn menu(){
    println!("Initialize Miner Node (1) or Client Node (2)? ");
    let opt=parse_input();
    let ops = operations();
    let client;
    if opt==1{
        client = create_client(None, true).await.expect("FAILURE CREATING CLIENT NODE").unwrap();
        println!("Node Started -> Id: {}", client.clone().info.unwrap().id);
        loop {
            println!("What to do? ");
            for i in 0..ops.len(){println!("\t{}({i})", ops.get(i).unwrap())}
            let op_n = parse_input();
            if op_n==0{
                let n = neighbours_request(client.clone(), format_url(client.clone().info.unwrap().ip, client.clone().info.unwrap().port.to_string())).await;
                for i in n{
                    println!("\t{}", i.id)
                }
            }
            else if op_n==1 {
                println!("How much to transfer? ");
                let val = parse_input();
                println!("Destination ID? ");
                let mut destination = String::new();
                io::stdin().read_line(&mut destination).expect("Failed to read line");
                destination = destination.trim().to_string();
                let n = neighbours_request(client.clone(), format_url(client.clone().info.unwrap().ip, client.clone().info.unwrap().port.to_string())).await;
                for i in n{
                    let url = format_url(i.ip, i.port.to_string());
                    transaction_request(client.clone(), client.info.clone().unwrap().id, val, destination.clone(), url).await;
                }
            }
            else if op_n==2 {  }
            else if op_n==3 {  }
            else if op_n==4 {  }
            else if op_n==5 {  }
            else if op_n==6 {  }
        }
    }
    else if opt==2 {
        client = create_client(None, false).await.expect("FAILURE CREATING CLIENT NODE").unwrap();
        println!("What to do? ");
        for i in 0..ops.len(){println!("\t{}({i})", ops.get(i).unwrap())}
    }
    else { println!("Unknown Option, Please Try Again!") }
}

#[tokio::main]
async fn main(){
    let token = CancellationToken::new();
    let cloned_token = token.clone();
    tokio::select! {
        _ = cloned_token.cancelled() => {
            println!("CLOSING");
        }
        _ = tokio::time::sleep(Duration::from_secs(0)) => {tokio::spawn(async move{
            loop{menu().await;}
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