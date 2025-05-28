use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Getters)]
#[getset(get = "pub")]
pub struct IndexInfo {
    pub cluster_name: String,
    pub index_name: String,
}
