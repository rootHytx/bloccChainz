use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration};
use tokio::task;
use tonic::{Status};
use tonic::transport::Server;
use crate::endpoint::EndpointService;
use crate::proto::*;
use crate::requests::*;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;

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

pub async fn init_client(node: Node) ->  Result<bool, Status>{
    if node.info.clone().unwrap().bootstrap && node.info.clone().unwrap().port== BOOTSTRAP_PORTS.get(0).unwrap().parse::<u32>().unwrap(){
        return Ok(true)
    }
    let neighbours = join_request(node.clone()).await;
    if neighbours.len()==0{return Ok(false)};
    for i in neighbours.clone(){
        let mut send=Vec::new();
        send.push(node.info.clone().unwrap());
        update_request(node.clone(), send, format_url(i.ip, i.port.to_string())).await;
    }
    update_request(node.clone(), neighbours.clone(), format_url(node.info.clone().unwrap().ip, node.info.clone().unwrap().port.to_string())).await;
    Ok(true)
}

/*pub async fn remove_node(node: NodeInfo, url: String){
    let neighbours = neighbours_request(node.id.clone(), url.clone()).await;
    for i in neighbours{
        let url = format_url(i.clone().ip, i.clone().port.to_string());
        remove_request(node.clone(), url).await;
    }
}*/

pub async fn serve_client(service: EndpointService,addr: SocketAddr) -> Result<bool, Box<dyn std::error::Error>>{
    //println!("{}", format_url(addr.ip().to_string(), addr.port().to_string()));
    let msg= format!("FAILURE INITIALIZING NODE SERVER: {}", addr.clone());
    Server::builder()
        .add_service(crate::EndpointServer::new(service))
        .serve(addr.clone()).await.expect(msg.as_str());
    Ok(true)
}
pub async fn refresh(node:Node){
    tokio::time::sleep(Duration::new(REFRESH_PERIOD as u64, 0)).await;
    let url = format_url(node.info.clone().unwrap().ip, node.info.clone().unwrap().port.to_string());
    let neighbours = neighbours_request(node.clone(), url.clone()).await;
    let mut res = 0;
    for i in neighbours.clone(){
        let receiver_url = format_url(i.ip.clone(), i.port.clone().to_string());
        if !ping_request(receiver_url).await{
            println!("NODE {} IS DOWN, REMOVING...", i.id.clone());
            remove_request(node.clone(), i.clone(), url.clone()).await;
        }
        else { res+=1 }
        //else { if res=="".to_string(){res = format!("{}@{}",i.id, i.port)}else{res = format!("{}, {}@{}", res, i.id, i.port)} };
    }
    println!("NODE {}@{} ACTIVE NEIGHBOURS: {}", node.info.clone().unwrap().id, node.info.clone().unwrap().port, res);
}

pub async fn create_client(port:Option<u32>, miner:bool) -> Result<Option<Node>, Box<dyn std::error::Error>>{
    let mut service = EndpointService::default();
    let node;
    let addr= get_ip_address().await;
    if port!=None{ node = service.setup_client(addr, port, false).await }
    else{ node = service.setup_client(addr, None, miner).await; };
    //println!("CREATING NODE: {}@{}", node.info.clone().unwrap().id, node.info.clone().unwrap().port);
    let server_node = node.clone();
    task::spawn(async move{
        let addr:SocketAddr = format_addr(server_node.info.clone().unwrap().ip, server_node.info.clone().unwrap().port.to_string());
        println!("ADDRESS: {}", addr.clone());
        serve_client(service, addr).await.expect("FAILURE SPAWNING CLIENT SERVER");
    });
    if port!=None{
        let time_node=node.clone();
        task::spawn(async move{
            loop{refresh(time_node.clone()).await;}
        });
    };
    init_client(node.clone()).await.expect("FAILURE INITIALIZING CLIENT");
    Ok(Option::from(node.clone()))
}