use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct SlackConfig {
    pub bot_token: String,
    pub channel: String,
}