use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::ClusterConfig::*;
use crate::model::TeleBot::*;

use crate::repository::es_repository::*;

/* 
    Elasticsearch DB 초기화
*/
pub fn initialize_db_clients(es_info_path: &str) -> Result<Vec<EsRepositoryPub>, anyhow::Error> {

    let mut elastic_conn_vec: Vec<EsRepositoryPub> = Vec::new();
    
    let cluster_config: ClusterConfig = read_json_from_file::<ClusterConfig>(es_info_path)?;
    
    for config in &cluster_config.clusters {
        
        let es_helper = EsRepositoryPub::new(
            &config.cluster_name,
            config.hosts.clone(), 
            &config.es_id, 
            &config.es_pw)?;
        
        elastic_conn_vec.push(es_helper);
    }
    
    Ok(elastic_conn_vec)

}


/*
    Telebot 을 전역적으로 초기화 함.
*/
pub fn initialize_tele_bot_client(tele_info_path: &str) -> Result<(), anyhow::Error> {

    let tele_bot: Telebot = read_json_from_file::<Telebot>(tele_info_path)?;
    
    let mut telebot_guard = match TELE_BOT.write() {
        Ok(telebot_guard) => telebot_guard,
        Err(e) => return Err(anyhow!("[RWLock Error][bot_send()] Failed to read 'TELE_BOT' data. : {:?}",e))
    };

    *telebot_guard = Some(tele_bot);
    
    Ok(())
}
