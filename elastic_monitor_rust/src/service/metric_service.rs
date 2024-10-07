use crate::common::*;

use crate::model::MessageFormatter::MessageFormatter;
use crate::model::Indicies::*;

use crate::service::tele_bot_service::*;

use crate::repository::es_repository::*;

#[derive(Clone, Debug)]
pub struct MetricService {
    tele_service: TelebotService,
    elastic_obj: EsRepositoryPub
}


impl MetricService {

    pub fn new(elastic_obj: EsRepositoryPub) -> Self {
        
        let tele_service = TelebotService::new();
        let metric_service = MetricService{tele_service, elastic_obj};

        metric_service
    }
    
    /*
        Task 세트
    */
    pub async fn task_set(&self) -> Result<(), anyhow::Error> {

        // 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.
        match self.get_cluster_node_check().await {  
            Ok(flag) => {
                if !flag { return Ok(()) }
            },
            Err(e) => {
                error!("{:?}", e)
            }
        }
        
        // 2. 클러스터의 상태를 살핀다.
        let health_flag = self.get_cluster_health_check().await?;
        
        if health_flag {
            // 3. pending tasks 가 없는지 확인해준다.
            let _pending_task_res = self.get_cluster_pending_tasks().await?;
            
        } else {
            // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
            match self.get_cluster_unstable_index_infos().await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e)
                }
            }
        }
        
        Ok(())
    } 

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
                self.elastic_obj.cluster_name().to_string(), 
                conn_fail_hosts.join("\n"), 
                String::from("Elasticsearch Connection Failed"), 
                String::from("The connection of these hosts has been LOST."));
            
            let send_msg = msg_fmt.transfer_msg();
            self.tele_service.bot_send(send_msg.as_str()).await?;   
            
            info!("{:?}", msg_fmt);

            return Ok(false);
        }
        
        Ok(true)
    }


    /*
        Cluster 의 상태를 반환해주는 함수 -> green, yellow, red
    */
    async fn get_cluster_health_check(&self) -> Result<bool, anyhow::Error> {
        
        // 클러스터 상태 체크
        let cluster_status_json: Value = self.elastic_obj.get_health_info().await?;
        
        let cluster_status = cluster_status_json.get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("[Value Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?;
        
        if cluster_status != "green" {

            let msg_fmt = MessageFormatter::new(
                self.elastic_obj.cluster_name().to_string(), 
                String::from(""), 
                String::from("Elasticsearch Cluster health condition issue"), 
                String::from(format!("The current cluster health status is {}", cluster_status)));
            
            let send_msg = msg_fmt.transfer_msg();
            self.tele_service.bot_send(send_msg.as_str()).await?;   
            
            info!("{} cluster health status is {:?}", self.elastic_obj.cluster_name(), msg_fmt);
            
            return Ok(false)
        }
        
        Ok(true)
    }


    /*
        불안정한 인덱스들을 추출하는 함수
    */
    async fn get_cluster_unstable_index_infos(&self) -> Result<(), anyhow::Error> {

        let cluster_stat_resp = self.elastic_obj.get_indices_info().await?;
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
        
        let msg_fmt = MessageFormatter::new(
            self.elastic_obj.cluster_name().to_string(), 
            String::from(""), 
            String::from("Elasticsearch Index health condition issue"), 
            String::from(err_detail));
        
        let send_msg = msg_fmt.transfer_msg();
        self.tele_service.bot_send(send_msg.as_str()).await?;   
        
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

        
        println!("{:?}", pending_task);

        Ok(())
    }

}
