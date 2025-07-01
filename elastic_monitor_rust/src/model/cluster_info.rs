use crate::common::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub hosts: Vec<String>,
    pub es_id: String,
    pub es_pw: String,
    pub index_pattern: String,
    pub per_index_pattern: String,
    pub urgent_index_pattern: String,
}
