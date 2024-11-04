use crate::common::*;

use crate::model::SmtpJson::*;
use crate::model::ReceiverEmailList::*;
use crate::model::MessageFormatter::*;

use crate::utils_modules::io_utils::*;


#[doc = "전역 SMTP 통신 인스턴스를 선언"]
static SMTP_REPO: Lazy<Arc<SmtpRepositoryPub>> = Lazy::new(|| {
    initialize_smtp_clients()
});


#[doc = "smtp 통신 객체를 초기화해주는 함수"]
pub fn initialize_smtp_clients() -> Arc<SmtpRepositoryPub> {

    let smtp_info_path =  "./datas/smtp_info.json";
    let email_receiver_info = "./datas/email_receiver_info.json";
    
    let smtp_info_json: SmtpJson = match read_json_from_file::<SmtpJson>(smtp_info_path) {
        Ok(smtp_info_json) => smtp_info_json,
        Err(e) => {
            error!("[Error][initialize_smtp_clients()] Failed to object '{}': {:?}", smtp_info_path, e);
            panic!("{:?}", e)
        }
    };
    
    let receiver_email_list: ReceiverEmailList =  match read_json_from_file::<ReceiverEmailList>(email_receiver_info) {
        Ok(receiver_email_list) => receiver_email_list,
        Err(e) => {
            error!("[Error][initialize_smtp_clients()] Failed to object '{}' {:?}", email_receiver_info, e);
            panic!("{:?}", e)
        }
    };

    Arc::new(SmtpRepositoryPub::new(smtp_info_json, receiver_email_list))
}


#[doc = "TelebotService 를 Thread-safe 하게 이용하는 함수."]
pub fn get_smtp_repo() -> Arc<SmtpRepositoryPub> {
    Arc::clone(&SMTP_REPO)
}


#[async_trait]
pub trait SmtpRepository {
    async fn send_message_to_receiver_html(&self, email_id: &str, subject: &str, html_content: &str) -> Result<(), anyhow::Error>;
    async fn send_message_to_receivers(&self, send_email_form: &HtmlContents) -> Result<(), anyhow::Error>;
} 


#[derive(Serialize, Deserialize, Debug, Getters, new)]
#[getset(get = "pub")]
pub struct SmtpRepositoryPub {
    smtp_info_json: SmtpJson,
    receiver_email_list: ReceiverEmailList
}


#[async_trait]
impl SmtpRepository for SmtpRepositoryPub {
    
    #[doc = "수신자에게 html 형식의 이메일을 보내주는 함수"]
    async fn send_message_to_receiver_html(&self, email_id: &str, subject: &str, html_content: &str) -> Result<(), anyhow::Error> {

        let email = Message::builder()
            .from(self.smtp_info_json.credential_id.parse().unwrap())
            .to(email_id.parse().unwrap())
            .subject(subject)
            .multipart(
                MultiPart::alternative() 
                    .singlepart(
                        SinglePart::html(html_content.to_string())
                    )
            )?;

        let creds = Credentials::new(
            self.smtp_info_json.credential_id().to_string(), 
            self.smtp_info_json.credential_pw().to_string()
        );
        
        let mailer = 
            AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(self.smtp_info_json.smtp_name().as_str())?
                .credentials(creds)
                .build();
        
        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(anyhow!("{:?} : Failed to send email to {} ", e, email_id))
            }
        }
    }  
    
    
    #[doc = "지정된 수신자 모두에게 Error에 내용에 대한 이메일을 보내주는 함수"]
    async fn send_message_to_receivers(&self, html_contents: &HtmlContents) -> Result<(), anyhow::Error> {
        
        /* 이메일 수신자. */
        let receiver_email_list = self.receiver_email_list.receivers();
        
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
            let email_id = receiver.email_id();
            self.send_message_to_receiver_html(email_id.as_str(), "[Elasticsearch] Error Alert", &html_template)
        });
        
        let results = join_all(tasks).await;

        for result in results {
            match result {
                Ok(_) => info!("Email sent successfully"),
                Err(e) => error!("[Error][send_message_to_receivers()] Failed to send email: {}", e),
            }
        }
        
        Ok(())
    } 
}