use crate::common::*;

#[async_trait]
pub trait SqlServerRepository {
    async fn execute_imailer_procedure(
        &self,
        send_email: &str,
        email_subject: &str,
        email_content: &str,
    ) -> Result<(), anyhow::Error>;
    //async fn send_message_to_receiver_html(&self, email_format: &HtmlContents) -> Result<(), anyhow::Error>;
}
