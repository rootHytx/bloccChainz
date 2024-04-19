use std::string::ToString;

pub const ID_SIZE: usize = 5; //Size of the NODE_ID (truncate hash output, see node.rs)
pub const N_BUCKETS: usize = ID_SIZE*8; //For each bit of the NODE_ID, add one bucket (if ID_SIZE is in bytes, bytes*8 = bits)
pub const K_SIZE: usize = 5; //How many nodes per bucket
pub const N_BOOTSTRAPS: i32 = 3; //Number of bootstrap nodes
pub const IP_ADDRESS: &str = "127.0.0.1";