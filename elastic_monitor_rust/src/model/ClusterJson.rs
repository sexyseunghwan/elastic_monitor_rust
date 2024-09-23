use crate::common::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClusterJson {
    pub name: String,
    pub hosts: Vec<String>,
    pub username: String,
    pub password: String,
}