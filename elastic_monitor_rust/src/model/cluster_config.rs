use crate::common::*;
use crate::model::cluster_info::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterConfig {
    pub clusters: Vec<ClusterInfo>,
}
