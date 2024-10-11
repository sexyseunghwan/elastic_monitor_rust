use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct MetricInfo {
     pub host: String,
     pub jvm_usage: i64,
     pub cpu_usage: i64,
     pub disk_usage: i64, 
}