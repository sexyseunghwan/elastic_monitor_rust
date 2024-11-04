use crate::common::*;

use crate::utils_modules::io_utils::*;

// 전역 Telebot 인스턴스를 선언
static TELEGRAM_SERVICE: Lazy<Arc<TelebotServicePub>> = Lazy::new(|| {
    initialize_tele_bot_client()
});


/*
    Telebot 을 전역적으로 초기화 함.
*/
pub fn initialize_tele_bot_client() -> Arc<TelebotServicePub> {

    let tele_info_path = "./datas/tele_info.json";
    let tele_bot: TelebotServicePub = match read_json_from_file::<TelebotServicePub>(tele_info_path) {
        Ok(tele_bot) => tele_bot,
        Err(e) => {
            error!("[Error][initialize_tele_bot_client()] cannot read '{}' : {:?}", tele_info_path, e);
            panic!("{:?}",e)
        }
    };
    
    Arc::new(tele_bot)
}


/*
    TelebotService 를 Thread-safe 하게 이용하는 함수.
*/
pub fn get_telegram_service() -> Arc<TelebotServicePub> {
    Arc::clone(&TELEGRAM_SERVICE)
}


#[async_trait]
pub trait TelebotService {
    async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error>;
    async fn try_send(&self, client: &reqwest::Client, url: &str, body: &Value) -> Result<(), anyhow::Error>;
}

// TelebotService는 비즈니스 로직을 담당하는 서비스 레이어로 분리
#[derive(Clone, Debug, Deserialize, Serialize, new)]
pub struct TelebotServicePub {
    pub bot_token: String,
    pub chat_room_id: String,
}


#[async_trait]
impl TelebotService for TelebotServicePub {
   
   // Telegram bot 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생
   async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
       
       let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
       
       let body = serde_json::json!({
           "chat_id": self.chat_room_id,
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