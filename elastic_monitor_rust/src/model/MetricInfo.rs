use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
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
     pub os_swap_usage: f64
}