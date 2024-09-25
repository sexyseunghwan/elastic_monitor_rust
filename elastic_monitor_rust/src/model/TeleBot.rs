use crate::common::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Telebot {
    pub bot_token: String,
    pub chat_room_id: String,
}