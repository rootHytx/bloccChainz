use std::error::Error;
use std::io;
use std::net::{SocketAddr};
use openssl::rsa::Rsa;
use tokio::net::TcpSocket;
use crate::proto::NodeInfo;

pub const ID_SIZE: usize = 10; //Size of the NODE_ID (truncate hash output, see node.rs)
pub const N_BUCKETS: usize = ID_SIZE * 4; //For each bit of the NODE_ID, add one bucket (if ID_SIZE is in bytes, bytes*8 = bits)
pub const K_SIZE: usize = 5; //How many nodes per bucket
pub const BOOTSTRAP_PORTS: &'static[&str] = &["55555", "55556", "55557"]; // Static ports for server initialization
pub const REFRESH_PERIOD: i32 = 5;
pub const PREFIX_LENGTH: i32= 2;
pub const GENESIS: &str = "00f151242e0010e58cde0d6644d9db53a8552f0e2d26628c9a72199005b5a76e";
pub const TRANSACTION_NUMBER: i32 = 10;

pub fn format_url(ip:String, port:String) -> String{
    format!("http://{}:{}", ip, port)
}
pub fn format_addr(ip:String, port:String) -> SocketAddr{
    format!("{}:{}", ip, port).parse().unwrap()
}
pub fn parse_input() -> i32{
    let mut t = String::new();
    loop{
        io::stdin()
            .read_line(&mut t)
            .expect("Failed to read line");
        match t.trim().parse::<i32>() {
            Ok(nr) => return nr,
            Err(ref e) => println!("ERROR PARSING INTEGER {}", e),
        }
    };
}
pub fn new_key() -> (Vec<u8>, Vec<u8>){
    let keypair = Rsa::generate(1024).unwrap();
    (keypair.private_key_to_pem().unwrap(), keypair.public_key_to_pem().unwrap())
}
pub async fn get_ip_address() -> String{
    local_ip_address::local_ip().unwrap().to_string()
}
pub async fn known_bootstrap_addresses() -> Vec<String>{
    let mut res: Vec<String> = Vec::new();
    res.push(get_ip_address().await);
    res
}

pub fn bind(destination:String) -> Result<Option<TcpSocket>, Box<dyn Error>>{
        let socket = TcpSocket::new_v4();
        if socket.is_ok(){
            let res=socket.unwrap();
            res.set_reuseaddr(true).unwrap(); // allow to reuse the addr both for connect and listen
            res.set_reuseport(true).unwrap(); // same for the port
            res.bind(destination.parse().unwrap()).unwrap();
            return Ok(Option::from(res))
        }
        Ok(None)
    }

pub fn join_request_consensus(responses:Vec<Vec<NodeInfo>>) -> Vec<NodeInfo>{
    let mut seen = responses.clone();
    let mut most_votes:Vec<NodeInfo> = vec![];
    let mut nr_votes = 0;
    for i in responses{
      if seen.contains(&i.clone()){
          let n = seen.iter().filter(|&k| *k == i.clone()).count();
          if n>nr_votes{
              most_votes=i.clone();
              nr_votes=n;
          }
          seen.retain(|k| *k != i.clone());
      };
    };
    //println!("MOST: {:?}", most_votes.clone());
    most_votes
}

