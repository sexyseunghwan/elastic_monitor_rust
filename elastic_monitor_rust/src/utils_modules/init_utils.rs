use crate::common::*;
use crate::service::es_service::*;
use crate::utils_modules::io_utils::*;

use crate::model::ClusterConfig::*;

// 전역 Telebot 인스턴스를 선언
lazy_static! {
    static ref TELE_BOT: Arc<RwLock<Option<Telebot>>> = Arc::new(RwLock::new(None));
}



/*
    Elasticsearch DB connection 정보를 반환하는 함수
*/
pub async fn initialize_db_clients() -> Result<Vec<EsHelper>, anyhow::Error> {
    
    let file_path: &str = "./datas/server_info.json";
    
    let mut elastic_conn_vec: Vec<EsHelper> = Vec::new();

    let cluster_config: ClusterConfig = read_json_from_file::<ClusterConfig>(file_path)?;
    
    for config in &cluster_config.clusters {
        
        let es_helper = EsHelper::new(
            &config.cluster_name,
            config.hosts.clone(),
            &config.es_id,
            &config.es_pw,
        )?;

        elastic_conn_vec.push(es_helper);
    }
    
    Ok(elastic_conn_vec)
}