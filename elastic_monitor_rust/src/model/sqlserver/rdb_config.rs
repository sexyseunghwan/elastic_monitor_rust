use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct RdbConfig {
    pub host: String,
    pub port: String,
    pub user_id: String,
    pub user_pw: String,
    pub db_schema: String,
}
