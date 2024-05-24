use rand::Rng;
use tonic::{Status};
use tonic::transport::Channel;
use crate::proto;
use crate::proto::{ NeighboursRequest, Node, NodeInfo, UpdateRequest, Signature};
use crate::proto::endpoint_client::EndpointClient;
use crate::util::*;
use crate::signatures::*;
async fn try_connect(url:String) -> Result<Option<EndpointClient<Channel>>, Status>{
    if EndpointClient::connect(url.clone()).await.is_err(){return Ok(None)}
    Ok(Option::from(EndpointClient::connect(url).await.expect("FAILURE CONNECTING TO DESTINATION")))
}

pub async fn ping_request(url:String) -> bool{
    if let Some(_) = try_connect(url).await.expect("FAILURE PINGING DESTINATION"){ return true }
    false
}

pub async fn join_request(source:Node) -> Vec<NodeInfo>{
    let addresses = known_bootstrap_addresses().await;
    let ip_index= rand::thread_rng().gen_range(0..addresses.len());
    let url;
    if source.info.clone().unwrap().bootstrap{
        url = format_url(addresses.get(ip_index).unwrap().clone(), BOOTSTRAP_PORTS.get(0).unwrap().to_string());
    }
    else {
        let p_index = rand::thread_rng().gen_range(0..BOOTSTRAP_PORTS.len());
        url = format_url(addresses.get(ip_index).unwrap().clone(), BOOTSTRAP_PORTS.get(p_index).unwrap().to_string());
    };
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_join_request(source.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(proto::JoinRequest { node: Some(source.clone()), sign:Option::from(sign)});
        let response = client.join(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
        verify_join_response(response.get_ref().clone());
        return response.get_ref().clone().neighbours
    };
    Vec::new()
}

pub async fn find_node(source:Node, node:String, url:String) -> Option<NodeInfo> {
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_find_node_request(node.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(proto::FindNodeRequest{ target:node, sign:Option::from(sign)});
        let response = client.find_node(request).await.expect("FAILURE FINDING NODE");
        verify_find_node_response(response.get_ref().clone());
        return response.get_ref().clone().node
    };
    None
}

pub async fn remove_request(source:Node, node: NodeInfo, url:String){
    println!("REMOVING NODE: {}", node.id.clone());
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION") {
        let hash= sign_remove_request(node.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(proto::RemoveRequest{ node: Option::from(node.clone()), sign:Option::from(sign)});
        let response = client.remove_node(request).await.expect("FAILURE REMOVING NODES");
        verify_remove_response(response.get_ref().clone());
    }
}

pub async fn update_request(source:Node, nodes:Vec<NodeInfo>, url:String) -> bool{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_update_request(nodes.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(UpdateRequest{ neighbours: nodes.clone(), sign:Option::from(sign)});
        let response = client.update_node(request).await.expect("FAILURE UPDATING NODE");
        verify_update_response(response.get_ref().clone());
        return true
    };
    false
}

pub async fn neighbours_request(source:Node, url:String) -> Vec<NodeInfo>{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_neighbours_request(source.info.clone().unwrap().id, source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(NeighboursRequest{source_id:source.info.clone().unwrap().id, sign:Option::from(sign)});
        let response = client.get_neighbours(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
        verify_neighbours_response(response.get_ref().clone());
        return response.get_ref().clone().neighbours
    };
    Vec::new()
}