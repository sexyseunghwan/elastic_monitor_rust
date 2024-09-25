use std::panic::AssertUnwindSafe;

use crate::common::*;

use crate::service::es_service::*;

use crate::model::Indicies::*;


// /* 
//     불안정한 인덱스들을 추출하는 함수로 분리
// */ 
// async fn get_unstable_indices(cluster: &EsHelper) -> Result<Vec<Indicies>, anyhow::Error> {
    
//     let cluster_stat_resp = cluster.cluster_cat_indices_query().await?;
//     let unstable_indicies = cluster_stat_resp.trim().lines();

//     // 인덱스 상태 확인 및 벡터 생성
//     let mut indicies_vec = Vec::new();

//     for index in unstable_indicies {
//         let stats: Vec<&str> = index.split_whitespace().collect();
        
//         match stats.as_slice() {
//             [health, status, index, ..] if *health != "green" || *status != "open" => {
//                 indicies_vec.push(Indicies::new(health.to_string(), status.to_string(), index.to_string()));
//             }
//             [..] => continue, // 상태가 안정적인 경우는 무시
//         }
//     }

//     Ok(indicies_vec)
// }


// /*
//     Cluster 의 상태를 반환해주는 함수
// */
// pub async fn get_cluster_state(cluster: &EsHelper) -> Result<(bool, String, Vec<Indicies>), anyhow::Error> {
    
//     // 클러스터 상태 체크
//     let cluster_status_json = cluster.cluster_get_health_query().await?;
//     let cluster_name = cluster.cluster_name();

//     // 클러스터 상태 추출
//     let cluster_status = cluster_status_json.get("status")
//         .and_then(Value::as_str)
//         .ok_or_else(|| anyhow!("[Value Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?;
    
    
//     // 클러스터가 'green'이 아닌 경우에만 처리
//     let unstable_indicies_vec: Vec<Indicies> = if cluster_status != "green" {
//         get_unstable_indices(&cluster).await?
//     } else {
//         Vec::new()
//     };
    
//     Ok((cluster_status == "green", cluster_name.to_string(), unstable_indicies_vec))
// }


/*
    
*/
pub async fn get_cluster_node_check(cluster: &EsHelper) -> Result<(), anyhow::Error> {
    
    let conn_stats = cluster.cluster_get_ping_query().await;
    
    

    Ok(())
}

/*
    Cluster 의 상태를 반환해주는 함수 -> green, yellow, red
*/
pub async fn get_cluster_health_check(cluster: &EsHelper) -> Result<String, anyhow::Error> {
    
    // 클러스터 상태 체크
    let cluster_status_json = cluster.cluster_get_health_query().await?;
    
    let cluster_status = cluster_status_json.get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("[Value Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?;
    
    Ok(cluster_status.to_string())
}


/*
    불안정한 인덱스들을 추출하는 함수
*/
pub async fn get_cluster_unstable_index_infos(cluster: &EsHelper) -> Result<Vec<Indicies>, anyhow::Error> {

    let cluster_stat_resp = cluster.cluster_cat_indices_query().await?;
    let unstable_indicies = cluster_stat_resp.trim().lines();

    // 인덱스 상태 확인 및 벡터 생성
    let mut indicies_vec = Vec::new();

    for index in unstable_indicies {
        let stats: Vec<&str> = index.split_whitespace().collect();
        
        match stats.as_slice() {
            [health, status, index, ..] if *health != "green" || *status != "open" => {
                indicies_vec.push(Indicies::new(health.to_string(), status.to_string(), index.to_string()));
            }
            [..] => continue, // 상태가 안정적인 경우는 무시
        }
    }

    Ok(indicies_vec)
}