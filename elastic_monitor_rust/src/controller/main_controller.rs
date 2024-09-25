use crate::common::*;

use crate::utils_modules::init_utils::*;
use crate::utils_modules::io_utils::*;

use crate::service::es_service::*;
use crate::service::metric_service::*;
use crate::service::tele_bot_service::*;

use crate::model::Indicies::*;

pub async fn main_controller() {


    // 무한루프를 잡아놔서 계속 프로그램이 실행되도록 한다.
    loop {

        // Elasticsearch DB 커넥션 정보
        let db_infos_vec: Vec<EsHelper> = match initialize_db_clients().await {
            Ok(db_infos_vec) => db_infos_vec,
            Err(e) => {
                error!("{:?}", e);
                continue
            }
        };
        
        // Telebot connection 정보 인스턴스.
        let tele_bot = match read_json_from_file::<Telebot>("./datas/tele_info.json") {
            Ok(tele_bot) => tele_bot,
            Err(e) => {
                error!("{:?}", e);
                continue
            } 
        };  
        
        
        // 각 cluster 의 지표 탐색
        for cluster in db_infos_vec {
            
            // 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.
            
            // 2. 클러스터의 상태를 살핀다.
            
            // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
            
            // let cluster_metircs: (bool, String, Vec<Indicies>) = get_cluster_state(&cluster).await.unwrap();
            
            // let stable_yn: bool = cluster_metircs.0;
            
            // if !stable_yn {
                
            //     let cluster_name: String = cluster_metircs.1;
            //     let unstable_metircs: Vec<Indicies> = cluster_metircs.2;
                
            //     // bot 으로 처리
                        
            // } 

        }
           
        thread::sleep(Duration::from_secs(40));
    }
    
}