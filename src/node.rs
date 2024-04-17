use sha256::digest;
use rand::{RngCore};
const ID_SIZE: usize = 5;
use std::net::UdpSocket;

#[derive(Debug)]
pub struct Node{
    pub id: String,
    pub ip: String,
    pub port: u16,
}

impl Node{
    pub fn new(ip: String) -> Self{
        let mut input = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut input);
        let input = digest(&input);
        let destination = format!("{}:0", ip);
        //destination.push_str(":".to_string().push_str((node.port).to_string() as str));
        let socket = UdpSocket::bind(destination).expect("couldn't bind to address");
        Node{ id: input[..ID_SIZE].to_string(), ip, port: socket.local_addr().unwrap().port() }
    }
}