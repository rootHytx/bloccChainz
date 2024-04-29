use std::net::{SocketAddr, UdpSocket};
use tonic::{Response, Status};
use crate::proto;
use crate::proto::{JoinResponse, Node};
use crate::proto::network_client::NetworkClient;
pub const ID_SIZE: usize = 4; //Size of the NODE_ID (truncate hash output, see node.rs)
pub const N_BUCKETS: usize = ID_SIZE * 4; //For each bit of the NODE_ID, add one bucket (if ID_SIZE is in bytes, bytes*8 = bits)
pub const K_SIZE: usize = 5; //How many nodes per bucket
pub const N_BOOTSTRAPS: i32 = 3; //Number of bootstrap nodes
pub const IP_ADDRESS: &str = "127.0.0.1";
pub async fn init_client(mut node: Node, url: String) ->  Result<Response<JoinResponse>, Status>{
    let mut client = NetworkClient::connect(url).await.unwrap();
    let request = tonic::Request::new(proto::JoinRequest{ node: Some(node.clone())});
    let response = client.join(request).await?;
    Ok(response)
}
pub fn create_address() -> SocketAddr {
    let destination = format!("{}:0", IP_ADDRESS).to_string();
    let socket = UdpSocket::bind(destination).expect("couldn't bind to address");
    socket.local_addr().unwrap()
}