// use crate::common::*;


// #[derive(Clone, Serialize, Deserialize, Debug)]
// pub struct Telebot {
//     pub bot_token: String,
//     pub chat_room_id: String,
// }

// impl Telebot {
    
//     /*
            
//     */
//     pub async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
        
//         let mut try_cnt = 0;
        
//         while try_cnt < 3 {

//             let url: String = format!(
//                 "https://api.telegram.org/bot{}/sendMessage",
//                 self.bot_token
//             );

//             let body: Value = serde_json::json!({
//                 "chat_id": self.chat_room_id,
//                 "text": send_msg
//             });

//             let client: reqwest::Client = reqwest::Client::new();
//             let res = client.post(&url)
//                 .header("Content-Type", "application/json")
//                 .body(body.to_string())
//                 .send()
//                 .await;

//             match res {
//                 Ok(res) => {                    
//                     if !res.status().is_success() {
//                         let err_text = res.text().await.unwrap_or_else(|_| "Failed to get error message".to_string());
//                         error!("[Timeout Error][bot_send()] Failed to send message: {}. http communication retry begins after 40 seconds.", err_text);
//                         try_cnt += 1;
//                         thread::sleep(Duration::from_secs(40));
//                     } else {
//                         info!("Success send message");
//                         break;
//                     }
//                 },
//                 Err(err) => {
//                     error!("[Timeout Error][bot_send()] HTTP request failed: {}. http communication retry begins after 40 seconds.", err);
//                     try_cnt += 1;
//                     thread::sleep(Duration::from_secs(40));
//                 }
//             }
//         }

//         if try_cnt == 3 {
//             return Err(anyhow!("The system attempted to communicate with the telegram bot more than 3 times, but failed.".to_string()));
//         }
        
//         Ok(())
//     }


// }

use crate::common::*;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Telebot {
    pub bot_token: String,
    pub chat_room_id: String,
}

impl Telebot {
    
    
    // Telegram bot 이 메시지를 보내주는 기능 -> 3번 실패 시 에러발생
    pub async fn bot_send(&self, send_msg: &str) -> Result<(), anyhow::Error> {
        
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
