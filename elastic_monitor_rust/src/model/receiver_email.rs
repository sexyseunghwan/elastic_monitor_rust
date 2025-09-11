use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct ReceiverEmail {
    pub email_id: String,
}
