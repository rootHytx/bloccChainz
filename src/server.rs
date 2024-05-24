use std::time::Duration;
use proto::endpoint_server::*;
use util::*;
use requests::*;
use crate::signatures::*;
use blockchain::*;

mod endpoint;
mod util;
mod proto{
    tonic::include_proto!("kademlia");
}
mod node;
mod requests;
mod signatures;
mod blockchain;

async fn generate_bootstraps() -> Result<(), Box<dyn std::error::Error>>{
    create_client(Option::from(BOOTSTRAP_PORTS.get(0).unwrap().to_string().parse::<u32>().unwrap()), false).await.expect("FAILURE INITIALIZING BOOTSTRAP");
    tokio::time::sleep(Duration::new(0, 1000)).await;
    let mut boots= Vec::new();
    for mut i in 1..BOOTSTRAP_PORTS.len() {
        let port = BOOTSTRAP_PORTS.get(i).unwrap().to_string().parse::<u32>().unwrap();
        boots.push(create_client(Option::from(port.clone()), false).await.expect("FAILURE INITIALIZING BOOTSTRAP"));
    };
    for i in boots{
        let cur = i.unwrap().clone();
        let url = format_url(cur.info.clone().unwrap().ip, cur.info.clone().unwrap().port.to_string());
        let neighbours = neighbours_request(cur.clone(), url).await;
        println!("{:?}", neighbours);
    }
    Ok(())
}

#[tokio::main]
async fn main(){
    generate_bootstraps().await.expect("FAILED TO CREATE SERVER");
    loop{}
}