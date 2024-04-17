use std::error::Error;
use proto::kademlia_client::KademliaClient;
mod node;
use node::Node;

mod proto {
    tonic::include_proto!("kademlia");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    let url = "http://[::1]:50122";
    let node = Node::new("127.0.0.1".to_string());
    let mut client = KademliaClient::connect(url).await?;
    let request = tonic::Request::new(proto::ConnectRequest{ node_id : node.id, ip : node.ip, port: u32::from(node.port) });
    let response = client.connect_network(request).await?;
    println!("Response: {:?}", response.get_ref().nodes);
    Ok(())
}