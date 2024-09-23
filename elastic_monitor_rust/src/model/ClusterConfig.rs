use crate::common::*;
use crate::model::ClusterJson::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterConfig {
    pub clusters: Vec<ClusterJson>,
}