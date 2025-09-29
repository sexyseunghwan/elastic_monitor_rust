use crate::common::*;

#[async_trait]
pub trait TelebotRepository {
    async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error>;
    async fn try_send(&self, url: &str, body: &Value) -> Result<(), anyhow::Error>;
}