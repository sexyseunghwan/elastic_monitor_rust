use crate::common::*;

#[derive(Serialize, Deserialize, Debug, new)]
pub struct Indicies {
    pub health: String,
    pub status: String,
    pub index: String
}