use crate::common::*;

use crate::utils_modules::db_connection_utils::*;

use crate::service::es_service::*;
use crate::service::metric_service::*;

pub async fn main_controller() {

    // DB 커넥션
    let db_infos_vec: Vec<EsHelper> = match initialize_db_clients().await {
        Ok(db_infos_vec) => db_infos_vec,
        Err(e) => {
            error!("{:?}", e);
            panic!("{:?}", e)
        }
    };

    // 각 cluster 의 지표 탐색
    for cluster in db_infos_vec {
        
        get_cluster_state(cluster).await.unwrap();

    }




}