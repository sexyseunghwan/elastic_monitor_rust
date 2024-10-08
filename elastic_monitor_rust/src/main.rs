/*
Author      : Seunghwan Shin 
Create date : 2024-10-02 
Description : Elasticsearch 클러스터의 문제를 탐색하고 telegram 을 통해 문제를 전달해주는 서비스
    
History     : 2024-10-02 Seunghwan Shin       # first create
              2024-10-07 Seunghwan Shin       # Pending Task 모니터링 항목 추가.
              2024-10-00 Seunghwan Shin       # 
*/ 

mod common;
use common::*;

mod handler;
use handler::main_handler::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod service;
use service::metric_service::*;

mod model;
mod repository;
use repository::es_repository::*;


#[tokio::main]
async fn main() { 
    
    /* 전역 로거설정 */
    set_global_logger();
    
    info!("Start Program");

    // Elasticsearch DB 커넥션 정보
    let es_infos_vec: Vec<EsRepositoryPub> = match initialize_db_clients("./datas/server_info.json") {
        Ok(es_infos_vec) => es_infos_vec,
        Err(e) => {
            error!("[Error][main()] Cannot find json file: {:?}", e);
            panic!("{:?}", e)
        }
    };
    

    loop {

        // Handler 의존주입
        let mut handlers: Vec<MainHandler<MetricServicePub<EsRepositoryPub>>> = Vec::new();
        
        for cluster in es_infos_vec {
            let metirc_service = MetricServicePub::new(cluster);
            let maind_handler = MainHandler::new(metirc_service);
            handlers.push(maind_handler);
        }
        
        // Async 작업
        let futures = handlers.iter().map(|handler| {
            async move {                
                handler.task_set().await
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