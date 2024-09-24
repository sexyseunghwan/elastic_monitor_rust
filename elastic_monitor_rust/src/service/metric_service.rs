use crate::common::*;

use crate::service::es_service::*;


/*
    Cluster 의 상태를 체크해주는 함수
*/
pub async fn get_cluster_state(cluster: EsHelper) -> Result<(), anyhow::Error> {

    let cluster_stat_str = cluster.cluster_cat_indices_query().await?;

    let mut indicies_vec: Vec<Indicies> = Vec::new();
    let response_body: String = response.text().await?;
    let indicies: Vec<&str> = response_body.trim().split('\n').collect();
    
    for index in indicies {
        
        let stats: Vec<&str> = index.split_whitespace().collect();

        if let [health, status, index, ..] = stats.as_slice() {
            indicies_vec.push(Indicies::new(health.to_string(), status.to_string(), index.to_string()));
        } else {
            return Err(anyhow!("[Elasticsearch Error][node_cat_indices_query()] There is a problem with the variable 'stats'"));
        }
    }


    Ok(())
}
