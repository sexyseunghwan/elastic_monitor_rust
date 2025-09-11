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
    configs::{use_case_config::*, config::*}
};


#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct NotificationServicePub {
    pub email_list: ReceiverEmailList
}


impl NotificationServicePub {

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
                        "[Error][initialize_smtp_clients()] Failed to object '{}' {:?}",
                        email_receiver_info.to_string(),
                        e
                    );
                    panic!("{:?}", e)
                }
            };
        
        NotificationServicePub { email_list: receiver_email_list }

    }

    #[doc = "Telegram 을 통해서 문제를 전파해주는 함수"]
    async fn send_alarm_to_telegram<T: MessageFormatter + Sync + Send>(&self,  msg_fmt: &T) -> Result<(), anyhow::Error> {

        let tele_service: Arc<TelebotRepositoryPub> = get_telegram_repo();
        let telegram_format: String = msg_fmt.get_telegram_format();
        tele_service.bot_send(telegram_format.as_str()).await?;

        Ok(())
    }

    #[doc = "아이메일러를 통해서 문제를 전파해주는 함수"]
    async fn send_alarm_to_imailer<T: MessageFormatter + Sync + Send>(&self, msg_fmt: &T) -> Result<(), anyhow::Error> {

        let email_format: HtmlContents = msg_fmt.get_email_format();
        let sql_server_repo: Arc<SqlServerRepositoryPub> = get_sql_server_repo();
        
        /* html 파일 읽기 */
        let mut html_template: String = std::fs::read_to_string(&email_format.view_page_dir)?;
        
        /* 읽은 html을 기준으로 데이터 치환 */
        for (key, value) in &email_format.html_form_map {
            html_template = html_template.replace(&format!("{{{}}}", key), value)
        }

        let mail_subject: &str = "[Elasticsearch] Error Alert";

        for email in self.email_list().receivers() {
            
            match sql_server_repo.execute_imailer_procedure(email.email_id(), mail_subject, &html_template).await {
                Ok(_) => {
                    info!("Successfully sent email to {}", email.email_id());
                },
                Err(e) => {
                    error!(
                        "[ERROR][NotificationServicePub->send_alarm_to_imailer] Failed to send mail to {} : {:?}",
                        email.email_id(),
                        e
                    )
                }
            }
        }

        Ok(())
    }

}


#[async_trait]
impl NotificationService for NotificationServicePub {
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(
        &self,
        msg_fmt: &T,
    ) -> Result<(), anyhow::Error> {
        
        /* 현재 프로그램이 운영용/개발용인지 판단 */
        let use_case: Arc<UseCaseConfig> = get_usecase_config_info();

        if use_case.use_case == "prod" {
            /* Telegram 메시지 Send */
            self.send_alarm_to_telegram(msg_fmt).await?;
        }

        /* 개발/운영 환경 상관없이 아이메일러로 전송 (이메일) */
        // ============================ 잠시제거 ============================
        // ============================ 잠시제거 ============================
        self.send_alarm_to_imailer(msg_fmt).await?;
        // ============================ 잠시제거 ============================
        // ============================ 잠시제거 ============================
        
        Ok(())
    } 
}

