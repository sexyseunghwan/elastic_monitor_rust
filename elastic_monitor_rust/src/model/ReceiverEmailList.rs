use crate::common::*;

use crate::model::ReceiverEmail::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct ReceiverEmailList {
    pub receivers: Vec<ReceiverEmail>,
}
