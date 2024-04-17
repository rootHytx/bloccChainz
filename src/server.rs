use tonic::{Request, Response, Status};
use tonic::transport::Server;
use proto::kademlia_server::{Kademlia, KademliaServer};
use crate::proto::{PingRequest, PingResponse, ConnectRequest, ConnectResponse};
const N_BOOTSTRAPS: i32 = 4;
type BootstrapList = std::sync::Arc<tokio::sync::RwLock<Vec<Node>>>;

mod node;
use node::Node;
mod proto{
    tonic::include_proto!("kademlia");
}
#[derive(Debug, Default)]
struct KademliaService{
    bootstraps: BootstrapList,
}

impl KademliaService{
    async fn add_bootstrap(&self, new:Node) {
        self.bootstraps.write().await.push(new);
    }
    async fn init(&self) {
        for _i in 0..N_BOOTSTRAPS{
            self.add_bootstrap(Node::new("127.0.0.1".to_string())).await;
        }
    }
    async fn print_bootstraps(&self){
        println!("Current Bootstraps:\n{:?}", self.bootstraps.read().await)
    }
}

#[tonic::async_trait]
impl Kademlia for KademliaService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        println!("Got a request from {:?}", request.into_inner());
        let response = PingResponse{ message:"OK".to_string() };
        Ok(Response::new(response))
    }
    //noinspection ALL
    async fn connect_network(&self, request: Request<ConnectRequest>)  -> Result<Response<ConnectResponse>, Status> {
        println!("Got a request from {:?}", request.into_inner());
        let mut nodes = Vec::new();
        nodes.push("cona".to_string());
        nodes.push("conaça".to_string());
        let response = ConnectResponse{ nodes };
        Ok(Response::new(response))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let addr = "[::1]:50122".parse()?;
    let server = KademliaService::default();
    server.init().await;
    server.print_bootstraps().await;
    Server::builder()
        .add_service(KademliaServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}