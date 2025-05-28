use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub", set = "pub")]
pub struct MetricInfo {
    pub timestamp: String,
    pub host: String,
    pub jvm_usage: i64,
    pub cpu_usage: i64,
    pub disk_usage: i64,
    pub jvm_young_usage_byte: i64,
    pub jvm_old_usage_byte: i64,
    pub jvm_survivor_usage_byte: i64,
    pub query_cache_hit: f64,
    pub cache_memory_size: i64,
    pub os_swap_total_in_bytes: i64,
    pub os_swap_usage: f64,
    pub http_current_open: i64,
    pub node_shard_cnt: i64,
    pub indexing_latency: f64,
    pub query_latency: f64,
    pub fetch_latency: f64,
    pub translog_operation: i64,
    pub translog_operation_size: i64,
    pub translog_uncommited_operation: i64,
    pub translog_uncommited_operation_size: i64,
    pub flush_total: i64,
    pub refresh_total: i64,
    pub refresh_listener: i64,
}
