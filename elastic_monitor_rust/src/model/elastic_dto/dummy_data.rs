use crate::common::*;

#[derive(Debug, Serialize, Deserialize, new)]
pub struct DummyData {
    pub test: String,
}
