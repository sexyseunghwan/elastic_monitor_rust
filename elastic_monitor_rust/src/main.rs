/*
Author      : Seunghwan Shin 
Create date : 2024-10-02 
Description : Elasticsearch 클러스터의 문제를 탐색하고 telegram 을 통해 문제를 전달해주는 서비스
    
History     : 2024-10-02 Seunghwan Shin       # [v.1.0.0] first create
              2024-10-07 Seunghwan Shin       # [v.1.1.0] Pending Task 모니터링 항목 추가.
              2024-10-08 Seunghwan Shin       # [v.1.1.1] 소스코드에 추상화를 이용해서 아키텍쳐 적용
              2024-10-14 Seunghwan Shin       # [v.1.2.0] Pending Task 모니터링 제외.
              2024-10-17 Seunghwan Shin       # [v.1.3.0]
                                                1) 인덱스 삭제 알고리즘 제거
                                                2) jvm young, old, survivor 지표 모니터링 대상 추가
              2024-10-23 Seunghwan Shin       # [v.1.4.0] Elasticsearch 지표 모니터링 대상 추가 
              2024-11-04 Seunghwan Shin       # [v.1.5.0] Elasticsearch cluster 에 문제가 발생한 경우 이메일로도 알람을 받는 기능 추가.
              2024-11-25 Seunghwan Shin       # 1) [v.1.5.1] 설정 json 파일을 toml 파일로 전환
                                              # 2) comment(주석) 정리  
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

mod configs;


#[tokio::main]
async fn main() { 
    
    /* 전역 로거설정 */
    set_global_logger();
    
    info!("Start Program");

    /* Elasticsearch DB 커넥션 정보 */ 
    let es_infos_vec: Vec<EsRepositoryPub> = match initialize_db_clients() {
        Ok(es_infos_vec) => es_infos_vec,
        Err(e) => {
            error!("[Error][main()] Cannot find json file: {:?}", e);
            panic!("{:?}", e)
        }
    };
    
    /* Handler 의존주입 */ 
    let mut handlers: Vec<MainHandler<MetricServicePub<EsRepositoryPub>>> = Vec::new();
    
    for cluster in es_infos_vec {
        let metirc_service = MetricServicePub::new(cluster);
        let maind_handler = MainHandler::new(metirc_service);
        handlers.push(maind_handler);
    }
    
    
    loop {

        /* Async 작업 */ 
        let futures = handlers.iter().map(|handler| {
            async move {                
                handler.task_set().await /* 실제 Task */
            }
        });
        
        let results = join_all(futures).await;
        
        for result in results {
            match result {
                Ok(_) => {
                    info!("Program processed successfully");
                }
                Err(e) => {
                    error!("[Error][main()] Error processing template: {:?}", e);
                }
            }
        }   
        
        break; /* Test code */
        info!("Pending Program...");
        //std::thread::sleep(Duration::from_secs(10)); /* 10초 마다 탐색 -> 무한루프가 돌고 있으므로. */
    }
}