use crate::common::*;


#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct MonElasticConfig {
    pub cluster_name: String,
    pub hosts: Vec<String>,
    pub es_id: String,
    pub es_pw: String,
    pub pool_cnt: usize,
    pub index_pattern: String,
    pub per_index_pattern: String,
    pub urgent_index_pattern: String,
    
}