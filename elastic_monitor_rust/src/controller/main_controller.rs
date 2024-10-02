use crate::common::*;

use crate::utils_modules::init_utils::*;
use crate::utils_modules::logger_utils::*;

use crate::service::metric_service::*;

use crate::repository::es_repository::*;

/* 작업 SET */
async fn task_set(metirc_service: MetricService) -> Result<(), anyhow::Error> {
    
    
    // 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.
    match metirc_service.get_cluster_node_check().await {  
        Ok(flag) => {
            if !flag { return Ok(()) }
        },
        Err(e) => {
            error!("{:?}", e)
        }
    }
    
    
    // 2. 클러스터의 상태를 살핀다.
    match metirc_service.get_cluster_health_check().await {
        Ok(flag) => {
            if flag { return Ok(()) }
        },
        Err(e) => {
            error!("{:?}", e)
        }
    }
    
    // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
    match metirc_service.get_cluster_unstable_index_infos().await {
        Ok(_) => (),
        Err(e) => {
            error!("{:?}", e)
        }
    }

    Ok(())

}

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
            
            let service: MetricService = service.clone();

            async move {                
                task_set(service).await
            }
        });
        
        join_all(futures).await;

        break;        
        //thread::sleep(Duration::from_secs(60)); //60초 마다 탐색 -> 무한루프가 돌고 있으므로.
    }
    
}