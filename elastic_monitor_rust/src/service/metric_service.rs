use crate::common::*;

use crate::model::MessageFormatter::MessageFormatter;
use crate::model::Indicies::*;

use crate::service::tele_bot_service::*;

use crate::repository::es_repository::*;

#[async_trait]
pub trait MetricService {
    async fn get_cluster_node_check(&self) -> Result<bool, anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(&self, cluster_status: &str) -> Result<(), anyhow::Error>;
    async fn get_cluster_pending_tasks(&self) -> Result<(), anyhow::Error>;
}


#[derive(Clone, Debug)]
pub struct MetricServicePub<R: EsRepository> {
    elastic_obj: R
}

impl<R: EsRepository> MetricServicePub<R> {
    
    pub fn new(elastic_obj: R) -> Self {
        
        let metric_service = MetricServicePub{elastic_obj};
        metric_service
    } 
}


#[async_trait]
impl<R: EsRepository + Sync + Send> MetricService for MetricServicePub<R> {
    
    /*
        Elasticsearch 클러스터 내의 각 노드의 상태를 체크해주는 함수
    */
    async fn get_cluster_node_check(&self) -> Result<bool, anyhow::Error> {
        
        let conn_stats = self.elastic_obj.get_node_conn_check().await;

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
                self.elastic_obj.get_cluster_name(), 
                conn_fail_hosts.join("\n"), 
                String::from("Elasticsearch Connection Failed"), 
                String::from("The connection of these hosts has been LOST."));
            
            let send_msg = msg_fmt.transfer_msg();
            let tele_service = get_telegram_service();
            tele_service.bot_send(send_msg.as_str()).await?;   
            
            info!("{:?}", msg_fmt);

            return Ok(false);
        }
        
        Ok(true)
    }
    

    /*
        Cluster 의 상태를 반환해주는 함수 -> green, yellow, red
    */
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error> {
        
        // 클러스터 상태 체크
        let cluster_status_json: Value = self.elastic_obj.get_health_info().await?;
        
        let cluster_status = cluster_status_json.get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("[Value Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?
            .to_uppercase();
        
        Ok(cluster_status)
    }

    
    /*
        불안정한 인덱스들을 추출하는 함수
    */
    async fn get_cluster_unstable_index_infos(&self, cluster_status: &str) -> Result<(), anyhow::Error> {

        let cluster_stat_resp = self.elastic_obj.get_indices_info().await?;
        let unstable_indicies = cluster_stat_resp.trim().lines();
        
        // 인덱스 상태 확인 및 벡터 생성
        let mut indicies_vec: Vec<Indicies> = Vec::new();

        for index in unstable_indicies {
            let stats: Vec<&str> = index.split_whitespace().collect();
            
            match stats.as_slice() {
                [health, status, index, ..] if *health != "green" || *status != "open" => {
                    indicies_vec.push(Indicies::new(health.to_string().to_uppercase(), status.to_string().to_uppercase(), index.to_string()));
                }
                [..] => continue, // 상태가 안정적인 경우는 무시
            }
        }

        let mut err_detail = String::new();

        for index in indicies_vec {
            err_detail += index.get_indicies().as_str();
        }
        
        let msg_fmt = MessageFormatter::new(
            self.elastic_obj.get_cluster_name(), 
            self.elastic_obj.get_cluster_all_host_infos(), 
            String::from(format!("Elasticsearch Cluster health is [{}]", cluster_status)),
            String::from(err_detail));
        
        let send_msg = msg_fmt.transfer_msg();
        let tele_service = get_telegram_service();
        tele_service.bot_send(send_msg.as_str()).await?;    
        
        info!("{:?}", msg_fmt);
        
        Ok(())
    }

    
    /*
        중단된 작업 리스트를 확인해주는 함수
    */
    async fn get_cluster_pending_tasks(&self) -> Result<(), anyhow::Error> {

        let pending_task = self.elastic_obj
            .get_pendging_tasks()
            .await?;

        // tasks 필드가 배열로 존재하는지 확인
        let tasks = match pending_task["tasks"].as_array() {
            Some(tasks) if !tasks.is_empty() => tasks,
            _ => return Ok(()),  // tasks가 없거나, 빈 배열이면 작업 종료
        };

        // 모든 pending task를 문자열로 변환하여 한 번에 처리
        let task_details = tasks
            .iter()
            .map(|task| task.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        
        // 메세지 포맷 생성
        let msg_fmt = MessageFormatter::new(
            self.elastic_obj.get_cluster_name(), 
            String::new(),  // 빈 문자열
            "Elasticsearch pending task issue".to_string(),
            task_details
        );
        
        // 메세지 전송
        let send_msg = msg_fmt.transfer_msg();
        let tele_service = get_telegram_service();
        tele_service.bot_send(send_msg.as_str()).await?;  
        
        info!("{:?}", msg_fmt);

        Ok(())
    }
} 