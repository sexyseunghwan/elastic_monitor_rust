use crate::common::*;

#[derive(Debug, Serialize, Deserialize, Getters, new)]
#[getset(get = "pub")]
pub struct ErrorLogInfo {
    pub cluster_name: String,
    pub host: String,
    pub index_name: String,
    pub timestamp: String,
    pub err_title: String,
    pub err_detail: String,
}