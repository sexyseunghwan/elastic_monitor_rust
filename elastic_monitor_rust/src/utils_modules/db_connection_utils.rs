use crate::common::*;
use crate::service::es_service::*;
use crate::utils_modules::io_utils::*;

use crate::model::ClusterConfig::*;


/*
    Function that initializes db connection to a 'single tone'
*/
pub async fn initialize_db_clients() -> Result<> {
    
    let file_path = "./datas/server_info.json";
    
    let cluster_config_vec = match read_json_from_file::<ClusterConfig>(file_path) {
        Ok(cluster_config_vec) => cluster_config_vec,
        Err(e) => {
            error!("{:?}", e);
            panic!("{:?}", e)
        }
    };

        
    
    //let es_host: Vec<String> = env::var("ES_DB_URL").expect("[ENV file read Error][initialize_db_clients()] 'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    //let es_id = env::var("ES_ID").expect("[ENV file read Error][initialize_db_clients()] 'ES_ID' must be set");
    //let es_pw = env::var("ES_PW").expect("[ENV file read Error][initialize_db_clients()] 'ES_PW' must be set");
    
    // Elasticsearch connection
    // let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
    //     Ok(es_client) => es_client,
    //     Err(err) => {
    //         error!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
    //         panic!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
    //     }
    // };
    
    // let _ = ELASTICSEARCH_CLIENT.set(Arc::new(es_client));
    
    
}