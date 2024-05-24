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
use crate::proto::{NodeInfo, BucketNode, KBucket, Node, JoinResponse};
use crate::proto::endpoint_server::EndpointServer;
use crate::signatures::*;
use tokio_util::sync::CancellationToken;
use blockchain::*;
mod endpoint;
mod util;
mod node;
mod proto {
    tonic::include_proto!("kademlia");
}
mod blockchain;
mod requests;
mod signatures;
fn entry(n:i32) -> String{
    let mut number = String::new();
    loop{
        if n==0{println!("Create how many miner nodes? ")}
        else{println!("Create how many client nodes? ")}
        io::stdin()
            .read_line(&mut number)
            .expect("Failed to read line");
        match number.trim().parse::<i32>() {
            Ok(nr) => break,
            Err(ref e) => println!("ERROR PARSING INTEGER {}", e),
        }
    };
    number
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
            for i in 0..client_number.trim().parse::<i32>().unwrap(){
                res.push(create_client(None, false).await.expect("FAILURE INITIALIZING CLIENT"));
            };
            let finder=res.get(rand::thread_rng().gen_range(0..res.len())).unwrap().clone().unwrap();
            let tofind = res.get(rand::thread_rng().gen_range(0..res.len())).unwrap().clone().unwrap();
            let req = find_node(finder.clone(), tofind.info.clone().unwrap().id, format_url(finder.info.clone().unwrap().ip, finder.info.clone().unwrap().port.to_string())).await;
            println!("{} FOUND {:?}",finder.info.clone().unwrap().id, req.unwrap());
            for i in res{
                println!("{}", neighbours_request(i.clone().unwrap(), format_url(i.clone().unwrap().info.unwrap().ip, i.clone().unwrap().info.unwrap().port.to_string())).await.len())
            }
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