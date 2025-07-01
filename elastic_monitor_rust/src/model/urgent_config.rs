use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct UrgentConfig {
    pub metric_name: String,
    pub limit: f64,
}

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct UrgentConfigList {
    pub urgent: Vec<UrgentConfig>,
}
