use crate::common::*;

#[derive(Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub")]
pub struct SmtpConfig {
    pub smtp_name: String,
    pub credential_id: String,
    pub credential_pw: String,
}
