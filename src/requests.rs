use rand::Rng;
use tonic::{Status};
use tonic::transport::Channel;
use crate::proto;
use crate::proto::{NeighboursRequest, Node, NodeInfo, UpdateRequest, Signature, Block, RetrieveBlockchainResponse, RetrieveBlockchainRequest, TransactionRequest, MineRequest, UpdateBlockchainRequest, ObtainTransactionsRequest};
use crate::proto::endpoint_client::EndpointClient;
use crate::proto::miner_client::MinerClient;
use crate::util::*;
use crate::signatures::*;
async fn try_connect(url:String) -> Result<Option<EndpointClient<Channel>>, Status>{
    if EndpointClient::connect(url.clone()).await.is_err(){return Ok(None)}
    Ok(Option::from(EndpointClient::connect(url).await.expect("FAILURE CONNECTING TO DESTINATION")))
}
async fn try_connect_miner(url:String) -> Result<Option<MinerClient<Channel>>, Status>{
    if MinerClient::connect(url.clone()).await.is_err(){return Ok(None)}
    Ok(Option::from(MinerClient::connect(url).await.expect("FAILURE CONNECTING TO DESTINATION")))
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
        if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
            let hash = sign_join_request(source.clone(), source.skey.clone());
            let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
            let request = tonic::Request::new(proto::JoinRequest { node: Some(source.clone()), sign:Option::from(sign)});
            let response = client.join(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
            verify_join_response(response.get_ref().clone());
            return response.get_ref().clone().neighbours;
        };
    }
    else {
        let mut boot_addresses = Vec::new();
        let mut responses = Vec::new();
        for i in addresses{
            for j in 0..BOOTSTRAP_PORTS.len(){
                boot_addresses.push(format_url(i.clone(), BOOTSTRAP_PORTS.get(j).unwrap().to_string()));
            }
        }
        for url in boot_addresses{
            if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
                let hash = sign_join_request(source.clone(), source.skey.clone());
                let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
                let request = tonic::Request::new(proto::JoinRequest { node: Some(source.clone()), sign:Option::from(sign)});
                let response = client.join(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
                verify_join_response(response.get_ref().clone());
                responses.push(response.get_ref().clone().neighbours)
            };
        }
        return join_request_consensus(responses);
    };
    Vec::new()
}

pub async fn find_node(source:Node, node:String, url:String) -> Option<NodeInfo> {
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_find_node_request(source.info.clone().unwrap().id, node.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(proto::FindNodeRequest{ source_id:source.info.clone().unwrap().id, target:node, sign:Option::from(sign)});
        let response = client.find_node(request).await.expect("FAILURE FINDING NODE");
        verify_find_node_response(response.get_ref().clone());
        return response.get_ref().clone().node
    };
    None
}

pub async fn remove_request(source:Node, node: NodeInfo, url:String){
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION") {
        let hash= sign_remove_request(source.info.clone().unwrap().id, node.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(proto::RemoveRequest{ source_id:source.info.clone().unwrap().id, node: Option::from(node.clone()), sign:Option::from(sign)});
        let response = client.remove_node(request).await.expect("FAILURE REMOVING NODES");
        verify_remove_response(response.get_ref().clone());
    }
}

pub async fn update_request(source:Node, nodes:Vec<NodeInfo>, url:String) -> bool{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_update_request(source.info.clone().unwrap().id, nodes.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(UpdateRequest{ source_id:source.info.clone().unwrap().id, neighbours: nodes.clone(), sign:Option::from(sign)});
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

pub async fn retrieve_blockchain_request(source:Node, url:String) -> Vec<Block>{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_retrieve_blockchain_request(source.info.clone().unwrap().id, source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(RetrieveBlockchainRequest{source_id:source.info.clone().unwrap().id, sign:Option::from(sign)});
        let response = client.retrieve_blockchain(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
        verify_retrieve_blockchain_response(response.get_ref().clone());
        return response.get_ref().clone().blockchain
    }
    Vec::new()
}
pub async fn update_blockchain_request(source:Node, new:Block, url:String){
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_update_blockchain_request(source.info.clone().unwrap().id, new.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(UpdateBlockchainRequest{
            source_id:source.info.clone().unwrap().id,
            new:Option::from(new.clone()),
            sign:Option::from(sign)
        });
        let response = client.update_blockchain(request).await.expect("FAILURE UPDATING BLOCKCHAIN");
        verify_update_blockchain_response(response.get_ref().clone());
    }
}

pub async fn transaction_request(source:Node, sender:String, value:i32, destination:String, url:String) -> String{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_transaction_request(source.info.clone().unwrap().id, value.clone(), destination.clone(), source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(TransactionRequest{source_id:source.info.clone().unwrap().id,
            sender, value:value as u32, destination, sign:Option::from(sign)});
        let response = client.transaction(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
        verify_transaction_response(response.get_ref().clone());
        if response.get_ref().clone().state=="queued"{ println!("Transaction In Hold: Generating Block...") }
        return response.get_ref().clone().state
    }
    String::new()
}

pub async fn obtain_transactions_request(source:Node, url:String) -> Vec<String>{
    if let Some(mut client) = try_connect(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let hash = sign_obtain_transactions_request(source.info.clone().unwrap().id, source.skey.clone());
        let sign = Signature{hash, pkey:source.info.clone().unwrap().pkey};
        let request = tonic::Request::new(ObtainTransactionsRequest{source_id:source.info.clone().unwrap().id, sign:Option::from(sign)});
        let response = client.obtain_transactions(request).await.expect("FAILURE OBTAINING TRANSACTIONS");
        verify_obtain_transactions_response(response.get_ref().clone());
        return response.get_ref().clone().transactions;
    };
    Vec::new()
}

pub async fn mine_request(source:String, previous:Block, transactions:Vec<String>, url:String) -> Option<Block>{
    if let Some(mut client) = try_connect_miner(url).await.expect("FAILURE CONNECTING TO DESTINATION"){
        let request = tonic::Request::new(MineRequest{source_id:source, previous:Option::from(previous), transactions});
        let response = client.mine(request).await.expect("FAILURE RETRIEVING NEIGHBOURS");
        return response.get_ref().clone().new
    }
    None
}