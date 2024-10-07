use crate::common::*;

use crate::utils_modules::init_utils::*;
use crate::utils_modules::logger_utils::*;

use crate::service::metric_service::*;

use crate::repository::es_repository::*;


pub async fn main_controller() {

    set_global_logger(); /* 전역 로거설정 */
    
    info!("Start Program");

    // 프로그램이 실행되도록 한다.
    loop {
                
        // Elasticsearch DB 커넥션 정보
        let es_infos_vec: Vec<EsRepositoryPub> = match initialize_db_clients("./datas/server_info.json") {
            Ok(es_infos_vec) => es_infos_vec,
            Err(e) => {
                error!("{:?}", e);
                continue
            }
        };
        
        // Telebot connection 정보 인스턴스.
        match initialize_tele_bot_client("./datas/tele_info.json") {
            Ok(tele_bot) => tele_bot,
            Err(e) => {
                error!("{:?}", e);
                continue
            } 
        };  
        
        let mut services: Vec<MetricService> = Vec::new();
        
        for cluster in es_infos_vec {
            let metirc_service = MetricService::new(cluster);
            services.push(metirc_service);
        }
        
        
        let futures = services.iter().map(|service| {
            async move {                
                service.task_set().await
            }
        });
        
        let results = join_all(futures).await;

        for result in results {
            match result {
                Ok(_) => {
                    info!("Template processed successfully");
                }
                Err(e) => {
                    error!("Error processing template: {:?}", e);
                }
            }
        }
        

        break;        
        //thread::sleep(Duration::from_secs(60)); //60초 마다 탐색 -> 무한루프가 돌고 있으므로.
    }
    
}