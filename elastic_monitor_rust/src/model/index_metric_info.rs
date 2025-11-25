use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub", set = "pub")]
pub struct IndexMetricInfo {
    pub timestamp: String,
    pub index_name: String,
    pub translog_operation: i64,
    pub translog_operation_size: i64,
    pub translog_uncommited_operation: i64,
    pub translog_uncommited_operation_size: i64,
    pub flush_total: i64,
    pub refresh_total: i64,
    pub refresh_listener: i64,
}
