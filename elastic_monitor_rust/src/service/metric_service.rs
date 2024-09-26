use crate::common::*;

use crate::model::MessageFormatter::MessageFormatter;
use crate::service::es_service::*;
use crate::service::tele_bot_service::*;

use crate::model::Indicies::*;


/*
    Elasticsearch 클러스터 내의 각 노드의 상태를 체크해주는 함수
*/
pub async fn get_cluster_node_check(cluster: &EsHelper) -> Result<bool, anyhow::Error> {
    
    let conn_stats = cluster.get_cluster_conn_check().await;
    let tele_service = TelebotService::new();

    let conn_fail_hosts: Vec<String> = conn_stats
        .into_iter()
        .filter_map(|(es_host, is_success)| {
            if !is_success {
                Some(es_host)
            } else {
                None
            }
        })
        .collect();
    
    if !conn_fail_hosts.is_empty() {
        
        let msg_fmt = MessageFormatter::new(
            cluster.cluster_name().to_string(), 
            conn_fail_hosts.join("\n"), 
            String::from("Elasticsearch Connection Failed"), 
            String::from("The connection of these hosts has been LOST."));
        
        //let send_msg = msg_fmt.transfer_msg();
        //tele_service.bot_send(send_msg.as_str()).await?;   
        
        info!("{:?}", msg_fmt);

        return Ok(false);
    }
    
    Ok(true)
}


/*
    Cluster 의 상태를 반환해주는 함수 -> green, yellow, red
*/
pub async fn get_cluster_health_check(cluster: &EsHelper) -> Result<bool, anyhow::Error> {
    
    // 클러스터 상태 체크
    let cluster_status_json: Value = cluster.get_cluster_health().await?;
    
    let cluster_status = cluster_status_json.get("status")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("[Value Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?;
    
    if cluster_status != "green" {

        let tele_service = TelebotService::new();

        let msg_fmt = MessageFormatter::new(
            cluster.cluster_name().to_string(), 
            String::from(""), 
            String::from("Elasticsearch Cluster health condition issue"), 
            String::from(format!("The current cluster health status is {}", cluster_status)));
        
        //let send_msg = msg_fmt.transfer_msg();
        //tele_service.bot_send(send_msg.as_str()).await?;   
        
        //info!("{} cluster health status is {}", cluster.cluster_name(), send_msg);
        info!("{} cluster health status is {:?}", cluster.cluster_name(), msg_fmt);

        return Ok(false)
    }
    
    Ok(true)
}


/*
    불안정한 인덱스들을 추출하는 함수
*/
pub async fn get_cluster_unstable_index_infos(cluster: &EsHelper) -> Result<(), anyhow::Error> {

    let cluster_stat_resp = cluster.get_cluster_indices().await?;
    let unstable_indicies = cluster_stat_resp.trim().lines();
    
    // 인덱스 상태 확인 및 벡터 생성
    let mut indicies_vec: Vec<Indicies> = Vec::new();

    for index in unstable_indicies {
        let stats: Vec<&str> = index.split_whitespace().collect();
        
        match stats.as_slice() {
            [health, status, index, ..] if *health != "green" || *status != "open" => {
                indicies_vec.push(Indicies::new(health.to_string(), status.to_string(), index.to_string()));
            }
            [..] => continue, // 상태가 안정적인 경우는 무시
        }
    }

    let mut err_detail = String::new();

    for index in indicies_vec {
        err_detail += index.get_indicies().as_str();
    }

    let tele_service = TelebotService::new();
    let msg_fmt = MessageFormatter::new(
        cluster.cluster_name().to_string(), 
        String::from(""), 
        String::from("Elasticsearch Index health condition issue"), 
        String::from(err_detail));
    
    //let send_msg = msg_fmt.transfer_msg();
    //tele_service.bot_send(send_msg.as_str()).await?;   
    
    info!("{:?}", msg_fmt);
    
    Ok(())
}