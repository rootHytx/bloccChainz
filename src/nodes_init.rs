use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration};
use tokio::task;
use tonic::{Status};
use tonic::transport::Server;
use crate::endpoint::EndpointService;
use crate::proto::*;
use crate::requests::*;
use crate::util::*;
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
    if miner{

    }
    Ok(Option::from(node.clone()))
}