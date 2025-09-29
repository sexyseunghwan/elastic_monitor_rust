use crate::common::*;

use crate::model::configs::config::*;
use crate::model::configs::slack_config::*;

use crate::traits::repository::slack_repository::*;

/* 전역 Slack 인스턴스를 선언 */
static SLACK_REPO: once_lazy<Arc<SlackRepositoryImpl>> =
    once_lazy::new(initialize_slack_client);

/* Client 를 전역적으로 사용하기 위한 변수 선언 */
static REQ_CLIENT: once_lazy<Client> = once_lazy::new(Client::new);

#[doc = "Slack 을 전역적으로 초기화 함."]
pub fn initialize_slack_client() -> Arc<SlackRepositoryImpl> {
    let slack_info_config: Arc<SlackConfig> = get_slack_config_info();
    let slack_repo: SlackRepositoryImpl = SlackRepositoryImpl::new(
        slack_info_config.bot_token().to_string(),
        slack_info_config.channel().to_string(),
    );

    Arc::new(slack_repo)
}

#[doc = "SlackRepository 를 Thread-safe 하게 이용하는 함수."]
pub fn get_slack_repo() -> Arc<SlackRepositoryImpl> {
    Arc::clone(&SLACK_REPO)
}

#[derive(Clone, Debug, Deserialize, Serialize, new)]
pub struct SlackRepositoryImpl {
    pub bot_token: String,
    pub channel: String,
}

#[async_trait]
impl SlackRepository for SlackRepositoryImpl {
    #[doc = "Slack 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생"]
    async fn send_message(&self, message: &str) -> Result<(), anyhow::Error> {
        let url: String = "https://slack.com/api/chat.postMessage".to_string();

        let body: Value = serde_json::json!({
            "text": message,
            "channel": self.channel
        });

        // 최대 3번의 시도를 수행
        for try_cnt in 0..3 {
            match self.try_send(&url, &body).await {
                Ok(_) => {
                    info!("Successfully sent Slack message");
                    return Ok(());
                }
                Err(err) => {
                    error!(
                       "[Timeout Error][send_message()] Attempt {} failed: {}. Retrying in 40 seconds.",
                       try_cnt + 1,
                       err
                   );
                    sleep(Duration::from_secs(40)).await;
                }
            }
        }

        Err(anyhow!("[Timeout Error][send_message()] Failed to send message after 3 attempts to Slack."))
    }

    #[doc = "Slack으로 메시지 전송을 시도하는 내부 함수"]
    async fn try_send(&self, url: &str, body: &Value) -> Result<(), anyhow::Error> {
        let client: &once_lazy<Client> = &REQ_CLIENT;

        let res: reqwest::Response = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.bot_token))
            .body(body.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            let response_text = res.text().await?;
            let response_json: Value = serde_json::from_str(&response_text)?;

            if response_json["ok"].as_bool().unwrap_or(false) {
                Ok(())
            } else {
                let error_msg = response_json["error"].as_str().unwrap_or("Unknown error");
                Err(anyhow!("Slack API error: {}", error_msg))
            }
        } else {
            let err_text: String = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to retrieve error message".to_string());
            Err(anyhow!("HTTP request failed with status: {:?}", err_text))
        }
    }
}
