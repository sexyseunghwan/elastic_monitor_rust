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
     pub jvm_survivor_usage_byte: i64 
}