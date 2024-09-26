use crate::common::*;

use crate::utils_modules::init_utils::*;

use crate::service::es_service::*;
use crate::service::metric_service::*;

pub async fn main_controller() {


    // 무한루프를 잡아놔서 계속 프로그램이 실행되도록 한다.
    loop {
        
        // Elasticsearch DB 커넥션 정보
        let db_infos_vec: Vec<EsHelper> = match initialize_db_clients("./datas/server_info.json") {
            Ok(db_infos_vec) => db_infos_vec,
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
        
        // 각 cluster 의 지표 탐색
        for cluster in db_infos_vec {
            
            // 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.
            match get_cluster_node_check(&cluster).await {  
                Ok(flag) => {
                    if !flag { continue }
                },
                Err(e) => {
                    error!("{:?}", e)
                }
            }
            
            // 2. 클러스터의 상태를 살핀다.
            match get_cluster_health_check(&cluster).await {
                Ok(flag) => {
                    if flag { continue }
                },
                Err(e) => {
                    error!("{:?}", e)
                }
            }
            
            // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
            match get_cluster_unstable_index_infos(&cluster).await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e)
                }
            }
        }

        break;
        //thread::sleep(Duration::from_secs(60)); //60초 마다 탐색
    }
    
}