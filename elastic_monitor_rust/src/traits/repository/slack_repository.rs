use crate::common::*;

#[async_trait]
pub trait SlackRepository {
    async fn send_message(&self, message: &str) -> Result<(), anyhow::Error>;
    async fn try_send(&self, url: &str, body: &Value) -> Result<(), anyhow::Error>;
}