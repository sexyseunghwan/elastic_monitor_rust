use crate::common::*;

use crate::model::configs::config::*;
use crate::model::configs::telegram_config::*;

/* 전역 Telebot 인스턴스를 선언 */
static TELEGRAM_REPO: once_lazy<Arc<TelebotRepositoryPub>> =
    once_lazy::new(initialize_tele_bot_client);

/* Client 를 전역적으로 사용하기 위한 변수 선언 */
static REQ_CLIENT: once_lazy<Client> = once_lazy::new(Client::new);

#[doc = "Telebot 을 전역적으로 초기화 함."]
pub fn initialize_tele_bot_client() -> Arc<TelebotRepositoryPub> {
    let tele_info_config: Arc<TelegramConfig> = get_telegram_config_info();
    let tele_repo: TelebotRepositoryPub = TelebotRepositoryPub::new(
        tele_info_config.bot_token().to_string(),
        tele_info_config.chat_room_id().to_string(),
    );

    Arc::new(tele_repo)
}

#[doc = "TelebotService 를 Thread-safe 하게 이용하는 함수."]
pub fn get_telegram_repo() -> Arc<TelebotRepositoryPub> {
    Arc::clone(&TELEGRAM_REPO)
}

#[async_trait]
pub trait TelebotRepository {
    async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error>;
    async fn try_send(&self, url: &str, body: &Value) -> Result<(), anyhow::Error>;
}

// TelebotService는 비즈니스 로직을 담당하는 서비스 레이어로 분리
#[derive(Clone, Debug, Deserialize, Serialize, new)]
pub struct TelebotRepositoryPub {
    pub bot_token: String,
    pub chat_room_id: String,
}

#[async_trait]
impl TelebotRepository for TelebotRepositoryPub {
    #[doc = "Telegram bot 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생"]
    async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
        let url: String = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let body: Value = serde_json::json!({
            "chat_id": self.chat_room_id,
            "text": send_msg
        });

        //let client: &once_lazy<Client> = &REQ_CLIENT;

        // 최대 3번의 시도를 수행
        for try_cnt in 0..3 {
            match self.try_send(&url, &body).await {
                Ok(_) => {
                    info!("Successfully sent message");
                    return Ok(());
                }
                Err(err) => {
                    error!(
                       "[Timeout Error][bot_send()] Attempt {} failed: {}. Retrying in 40 seconds.",
                       try_cnt + 1,
                       err
                   );
                    sleep(Duration::from_secs(40)).await;
                }
            }
        }

        Err(anyhow!("[Timeout Error][bot_send()] Failed to send message after 3 attempts to the Telegram bot."))
    }

    // 메시지를 직접 보내주는 함수
    async fn try_send(&self, url: &str, body: &Value) -> Result<(), anyhow::Error> {
        let client: &once_lazy<Client> = &REQ_CLIENT;

        let res: reqwest::Response = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err_text: String = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to retrieve error message".to_string());
            Err(anyhow!("HTTP request failed with status: {:?}", err_text))
        }
    }
}
