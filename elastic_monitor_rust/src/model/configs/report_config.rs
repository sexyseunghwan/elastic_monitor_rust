use crate::common::*;

#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct ReportConfig {
    pub enabled: bool,
    pub cron_schedule: String,
    pub img_path: String,
}