use tonic::{Request, Response, Status};
use crate::proto::endpoint_server::Endpoint;
use crate::proto::{Node, NodeInfo, UpdateRequest, UpdateResponse, GnRequest, GnResponse};
use crate::util::create_address;

type SafeNode = std::sync::Arc<tokio::sync::RwLock<Node>>;
#[derive(Clone,Debug,Default)]
pub struct EndpointService{
    node:SafeNode,
}

impl EndpointService{
    pub async fn setup(&mut self, ip: String, boot:bool) -> Node{
        let node = Node::new(ip, boot);
        self.node = SafeNode::from(tokio::sync::RwLock::from(node.clone()));
        node
    }
    pub async fn insert(&self, info:NodeInfo){
        self.node.write().await.new_route(info);
    }
}
#[tonic::async_trait]
impl Endpoint for EndpointService{
    async fn update_node(&self, request: Request<UpdateRequest>) -> Result<Response<UpdateResponse>, Status> {
        let mut neighbours = Vec::new();
        if let Some(req) = request.get_ref().info.clone() {
            let mut own = self.node.write().await;
            if let Some(info) = own.info.clone(){
                if req.bootstrap{neighbours = own.get_neighbours();}
                else if info.bootstrap {neighbours = own.get_closest_nodes(req.clone());};
                own.new_route(req.clone());
                let buckets = own.get_quantity();
                println!("NODE {} BUCKETS: {:?}", info.id, buckets);
            }
        }
        Ok(Response::new(UpdateResponse{ neighbours }))
    }
    async fn get_neighbours(&self, request: Request<GnRequest>) ->Result<Response<GnResponse>, Status>{
        Ok(Response::new(GnResponse{ neighbours:self.node.read().await.get_neighbours() }))
    }
}