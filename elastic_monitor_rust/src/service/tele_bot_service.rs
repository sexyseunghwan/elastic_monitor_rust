use crate::common::*;

use crate::model::TeleBot::*;

// TelebotService는 비즈니스 로직을 담당하는 서비스 레이어로 분리
#[derive(Clone, Debug, new)]
pub struct TelebotService;

impl TelebotService {
        
    fn tele_init(&self) -> Result<Telebot, anyhow::Error> {
        
         // 전역 TELE_BOT 상태를 안전하게 읽기 (match를 사용하여 처리)
         let telebot_guard = match TELE_BOT.read() {
            Ok(telebot_guard) => telebot_guard,
            Err(e) => return Err(anyhow!("[RWLock Error][bot_send()] Failed to read 'TELE_BOT' data. : {:?}",e))
        };
        
        let telebot: Telebot = match &*telebot_guard {
            Some(telebot) => telebot.clone(),
            None => {
                return Err(anyhow!("[Error][bot_send()] TELE_BOT is not initialized"));
            }
        };
        
        Ok(telebot)
    }
    
    // Telegram bot 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생
    pub async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
        
        let telebot = self.tele_init()?;
        
        let url = format!("https://api.telegram.org/bot{}/sendMessage", telebot.bot_token);
        
        let body = serde_json::json!({
            "chat_id": telebot.chat_room_id,
            "text": send_msg
        });

        let client = reqwest::Client::new();
        
        // 최대 3번의 시도를 수행
        for try_cnt in 0..3 {
            
            match self.try_send(&client, &url, &body).await {
                
                Ok(_) => {
                    info!("Successfully sent message");
                    return Ok(());
                },
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
    async fn try_send(
        &self,
        client: &reqwest::Client,
        url: &str,
        body: &Value,
    ) -> Result<(), anyhow::Error> {
        
        let res = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            let err_text = res.text().await.unwrap_or_else(|_| "Failed to retrieve error message".to_string());
            Err(anyhow!(
                "HTTP request failed with status: {:?}",
                err_text
            ))
        }
    }
}