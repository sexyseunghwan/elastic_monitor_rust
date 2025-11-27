use crate::common::*;

#[derive(Debug, Deserialize, Serialize, Clone, new)]
pub struct BreakerInfo {
    limit_size_in_bytes: u64,
    estimated_size_in_bytes: u64,
    tripped: u64,
}
