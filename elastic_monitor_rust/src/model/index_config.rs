use crate::common::*;
use crate::model::index_info::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexConfig {
    pub index: Vec<IndexInfo>,
}
