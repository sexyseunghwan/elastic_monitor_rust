use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct TelegramConfig {
    pub bot_token: String,
    pub chat_room_id: String,
}
