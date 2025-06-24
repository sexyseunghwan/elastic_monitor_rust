use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct ThreadPoolStat {
    node_name: String,
    name: String,
    active: u32,
    queue: u32,
    rejected: u32,
}