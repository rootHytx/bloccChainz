use std::error::Error;
use std::fmt::format;
use std::io;
//use tonic::transport::Channel;
use proto::kademlia_client::KademliaClient;
mod node;
use node::*;
mod constants;
use constants::*;
mod routing_table;
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
    let node = Node::new(IP_ADDRESS.to_string());
    let mut url = "".to_string();
    while url=="".to_string(){
        url = create_url();
    }
    println!("{}",url);
    let mut client = KademliaClient::connect(url).await?;
    let request = tonic::Request::new(proto::ConnectRequest{ node_id : node.info.id, ip : node.info.ip, port: u32::from(node.info.port), bootstrap:false });
    let response = client.connect_network(request).await?;
    println!("Response: {:?}", response.get_ref().nodes);
    Ok(())
}