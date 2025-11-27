use crate::common::*;

use crate::model::configs::config::*;
use crate::model::configs::smtp_config::*;
use crate::model::message_formatter_dto::message_formatter::*;
use crate::model::receiver_email::*;
use crate::model::receiver_email_list::*;

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

use crate::traits::repository::smtp_repository_trait::*;

#[doc = "전역 SMTP 통신 인스턴스를 선언"]
static SMTP_REPO: once_lazy<Arc<SmtpRepositoryPub>> = once_lazy::new(initialize_smtp_clients);

#[doc = "smtp 통신 객체를 초기화해주는 함수"]
pub fn initialize_smtp_clients() -> Arc<SmtpRepositoryPub> {
    let smtp_config: &SmtpConfig = get_smtp_config_info();
    let email_receiver_info: &once_lazy<String> = &EMAIL_RECEIVER_PATH;

    let receiver_email_list: ReceiverEmailList =
        match read_toml_from_file::<ReceiverEmailList>(email_receiver_info) {
            Ok(receiver_email_list) => receiver_email_list,
            Err(e) => {
                error!(
                    "[Error][initialize_smtp_clients()] Failed to object '{}' {:?}",
                    email_receiver_info.to_string(),
                    e
                );
                panic!("{:?}", e)
            }
        };

    Arc::new(SmtpRepositoryPub::new(
        smtp_config.smtp_name().to_string(),
        smtp_config.credential_id().to_string(),
        smtp_config.credential_pw().to_string(),
        receiver_email_list,
    ))
}

#[doc = "SMTP를 Thread-safe 하게 이용하는 함수."]
pub fn get_smtp_repo() -> Arc<SmtpRepositoryPub> {
    Arc::clone(&SMTP_REPO)
}

#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct SmtpRepositoryPub {
    smtp_name: String,
    credential_id: String,
    credential_pw: String,
    receiver_email_list: ReceiverEmailList,
}

#[async_trait]
impl SmtpRepository for SmtpRepositoryPub {
    #[doc = "수신자에게 html 형식의 이메일을 보내주는 함수"]
    async fn send_message_to_receiver_html(
        &self,
        email_id: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), anyhow::Error> {
        let email: Message = Message::builder()
            .from(self.credential_id.parse().unwrap())
            .to(email_id.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(html_content.to_string())),
            )?;

        let creds: Credentials = Credentials::new(
            self.credential_id().to_string(),
            self.credential_pw().to_string(),
        );

        let mailer: AsyncSmtpTransport<lettre::Tokio1Executor> =
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(self.smtp_name().as_str())?
                .credentials(creds)
                .build();

        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id)),
        }
    }

    #[doc = "지정된 수신자 모두에게 Error에 내용에 대한 이메일을 보내주는 함수"]
    async fn send_message_to_receivers(
        &self,
        html_contents: &HtmlContents,
    ) -> Result<(), anyhow::Error> {
        /* 이메일 수신자. */
        let receiver_email_list: &Vec<ReceiverEmail> = self.receiver_email_list.receivers();

        /* html 파일 읽기 */
        let mut html_template: String = std::fs::read_to_string(&html_contents.view_page_dir)?;

        /* 읽은 html을 기준으로 데이터 치환 */
        for (key, value) in &html_contents.html_form_map {
            html_template = html_template.replace(&format!("{{{}}}", key), value)
        }

        /* Not Async */
        // for receiver in receiver_email_list {
        //     let email_id = receiver.email_id();
        //     self.send_message_to_receiver_html(email_id.as_str(), "[Elasticsearch] Index removed list", &html_content).await?;
        // }

        /* ASYNC TASK */
        let tasks = receiver_email_list.iter().map(|receiver| {
            let email_id: &String = receiver.email_id();
            self.send_message_to_receiver_html(
                email_id.as_str(),
                "[Elasticsearch] Error Alert",
                &html_template,
            )
        });

        let results: Vec<std::result::Result<(), anyhow::Error>> = join_all(tasks).await;

        for result in results {
            match result {
                Ok(_) => info!("Email sent successfully"),
                Err(e) => error!(
                    "[Error][send_message_to_receivers()] Failed to send email: {}",
                    e
                ),
            }
        }

        Ok(())
    }
}
