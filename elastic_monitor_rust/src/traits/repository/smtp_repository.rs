use crate::common::*;

use crate::model::message_formatter_dto::message_formatter::*;


#[async_trait]
#[allow(dead_code)]
pub trait SmtpRepository {
    async fn send_message_to_receiver_html(
        &self,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), anyhow::Error>;
    async fn send_message_to_receivers(
        &self,
        send_email_form: &HtmlContents,
    ) -> Result<(), anyhow::Error>;
}