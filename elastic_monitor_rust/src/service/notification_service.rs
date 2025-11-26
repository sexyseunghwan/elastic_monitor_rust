use crate::common::*;

use crate::repository::sql_server_repository::*;
use crate::repository::tele_bot_repository::*;

use crate::traits::notification_service_trait::*;
use crate::traits::sql_server_repository_trait::*;

use crate::env_configuration::env_config::*;

use crate::utils_modules::io_utils::*;

use crate::model::{
    message_formatter_dto::message_formatter::*,
    receiver_email_list::*, 
    configs::{use_case_config::*, config::*, smtp_config::*},
};


#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct NotificationServiceImpl {
    pub email_list: ReceiverEmailList
}


impl NotificationServiceImpl {

    pub fn new() -> Self {
        /* 개발환경/운영환경에 나눠서 생성자 생성방식 변경 */
        let use_case: Arc<UseCaseConfig> = get_usecase_config_info();

        let email_receiver_info: &once_lazy<String> = if use_case.use_case() == "prod" {
            &EMAIL_RECEIVER_PATH
        } else {
            &EMAIL_RECEIVER_DEV_PATH
        };

        let receiver_email_list: ReceiverEmailList =
            match read_toml_from_file::<ReceiverEmailList>(email_receiver_info) {
                Ok(receiver_email_list) => receiver_email_list,
                Err(e) => {
                    error!(
                        "[initialize_smtp_clients()] Failed to object '{}' {:?}",
                        email_receiver_info.to_string(),
                        e
                    );
                    panic!("{:?}", e)
                }
            };
        
        NotificationServiceImpl { email_list: receiver_email_list }

    }

    #[doc = "Telegram 을 통해서 문제를 전파해주는 함수"]
    async fn send_alarm_to_telegram<T: MessageFormatter + Sync + Send>(&self,  msg_fmt: &T) -> Result<(), anyhow::Error> {

        let tele_service: Arc<TelebotRepositoryPub> = get_telegram_repo();
        let telegram_format: String = msg_fmt.get_telegram_format();
        tele_service.bot_send(telegram_format.as_str()).await?;

        Ok(())
    }
    
    // #[doc = "Function that propagates issues via I-Mailer - for isolated networks"]
    // #[allow(dead_code)]
    // async fn send_alarm_to_imailer<T: MessageFormatter + Sync + Send>(&self, msg_fmt: &T) -> Result<(), anyhow::Error> {

    //     let email_format: HtmlContents = msg_fmt.get_email_format();
    //     let sql_server_repo: Arc<SqlServerRepositoryPub> = get_sql_server_repo();
        
    //     /* html 파일 읽기 */
    //     let mut html_template: String = std::fs::read_to_string(&email_format.view_page_dir)?;
        
    //     /* 읽은 html을 기준으로 데이터 치환 */
    //     for (key, value) in &email_format.html_form_map {
    //         html_template = html_template.replace(&format!("{{{}}}", key), value)
    //     }

    //     let mail_subject: &str = "[Elasticsearch] Error Alert";

    //     for email in self.email_list().receivers() {
            
    //         match sql_server_repo.execute_imailer_procedure(email.email_id(), mail_subject, &html_template).await {
    //             Ok(_) => {
    //                 info!("Successfully sent email to {}", email.email_id());
    //             },
    //             Err(e) => {
    //                 error!(
    //                     "[NotificationServiceImpl->send_alarm_to_imailer] Failed to send mail to {} : {:?}",
    //                     email.email_id(),
    //                     e
    //                 )
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    #[doc = "Function that propagates issues via I-Mailer - for isolated networks"]
    async fn send_alarm_to_imailer<T: MessageFormatter + Sync + Send>(
        &self,
        email_subject: &str,
        html_content: &str,
        receiver_email_list: ReceiverEmailList 
    ) -> anyhow::Result<()> {
        
        let sql_server_repo: Arc<SqlServerRepositoryPub> = get_sql_server_repo();

        for email in receiver_email_list.receivers() {
            match sql_server_repo.execute_imailer_procedure(email.email_id(), email_subject, &html_content).await {
                Ok(_) => {
                    info!("Successfully sent email to {}", email.email_id());
                },
                Err(e) => {
                    error!(
                        "[NotificationServiceImpl->send_alarm_to_imailer] Failed to send mail to {} : {:?}",
                        email.email_id(),
                        e
                    )
                } 
            }
        }

        Ok(())
    }
    
    #[doc = "Function that propagates issues via SMTP - for internet networks"]
    async fn send_message_to_smtp<T: MessageFormatter + Sync + Send>(
        &self,
        email_subject: &str,
        html_content: &str,
        receiver_email_list: ReceiverEmailList 
    ) -> anyhow::Result<()> {

        let smtp_config: Arc<SmtpConfig> = get_smtp_config_info();
        
        let tasks = receiver_email_list.receivers.iter().map(|receiver| {
                let email_id: &String = receiver.email_id();
                self.send_html_email_to_receiver(
                    &smtp_config,
                    email_id.as_str(),
                    email_subject,
                    html_content,
                )
            });

        let results: Vec<Result<String, anyhow::Error>> = join_all(tasks).await;

        for result in results {
            match result {
                Ok(succ_email_id) => info!("Email sent successfully: {}", succ_email_id),
                Err(e) => error!(
                    "[Error][send_message_to_receivers()] Failed to send email: {:?}",
                    e
                ),
            }
        }
        
        Ok(())
    }

     #[doc = r#"
        Asynchronous function that sends HTML format email to individual recipient.

        1. Creates email message object and sets sender/recipient/subject/body
        2. Creates Credentials object based on SMTP server authentication information
        3. Sets up connection to SMTP server through `AsyncSmtpTransport`
        4. Attempts actual email sending through configured mailer
        5. Returns recipient email address on successful sending, error on failure

        This function sends emails asynchronously using the lettre crate,
        and supports HTML multipart messages.

        # Arguments
        * `smtp_config` - SMTP server configuration information (includes server name, authentication info)
        * `email_id` - Recipient email address
        * `subject` - Email subject
        * `html_content` - HTML format email body

        # Returns
        * `Ok(String)` - Recipient email address on successful sending
        * `Err(anyhow::Error)` - On email composition or sending failure

        # Errors
        * Email address parsing failure
        * SMTP server connection failure
        * Authentication failure
        * Message transmission failure
    "#]
    async fn send_html_email_to_receiver(
        &self,
        smtp_config: &SmtpConfig,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<String, anyhow::Error> {
        let email: Message = Message::builder()
            .from(smtp_config.credential_id.parse()?)
            .to(email_id.parse()?)
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
            )?;

        let creds: Credentials = Credentials::new(
            smtp_config.credential_id().to_string(),
            smtp_config.credential_pw().to_string(),
        );

        let mailer: AsyncSmtpTransport<lettre::Tokio1Executor> =
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_config.smtp_name().as_str())?
                .credentials(creds)
                .build();

        match mailer.send(email).await {
            Ok(_) => Ok(email_id.to_string()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }

}


#[async_trait]
impl NotificationService for NotificationServiceImpl {
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(
        &self,
        msg_fmt: &T,
    ) -> Result<(), anyhow::Error> {
        
        /* 현재 프로그램이 운영용/개발용인지 판단 */
        // let use_case: Arc<UseCaseConfig> = get_usecase_config_info();

        // if use_case.use_case == "prod" {
        //     /* Telegram 메시지 Send */
        //     self.send_alarm_to_telegram(msg_fmt).await?;
        // }

        /* Send Message by the Telegram bot */
        self.send_alarm_to_telegram(msg_fmt).await?;

        /* Send message using iMailer */
        //self.send_alarm_to_imailer(msg_fmt).await?;
        
        /* Send messages using SMTP */
        

        Ok(())
    } 
}

