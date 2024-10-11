use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::model::MessageFormatter::MessageFormatter;
use crate::model::Indicies::*;
use crate::model::MetricInfo::*;

use crate::service::tele_bot_service::*;

use crate::repository::es_repository::*;

#[async_trait]
pub trait MetricService {
    async fn get_cluster_node_check(&self) -> Result<bool, anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(&self, cluster_status: &str) -> Result<(), anyhow::Error>;
    async fn get_cluster_pending_tasks(&self) -> Result<(), anyhow::Error>;
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error>;
    async fn delete_cluster_index(&self) -> Result<(), anyhow::Error>;
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


    /*
        각 cluster node 들의 정보를 elasticsearch 에 적재하는 함수
    */
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error> {

        let cluster_metrics: Value = self
            .elastic_obj
            .get_node_stats()
            .await?;
        

        if let Some(nodes) = cluster_metrics["nodes"].as_object() { 

            let cur_utc_time = get_current_utc_naivedate();

            // 날짜 기준으로 인덱스 이름 맵핑
            let index_name = format!("metric_info_{}", get_str_from_naivedate(cur_utc_time, "%Y%m%d"));
                        
            let mut metric_vec: Vec<MetricInfo> = Vec::new();

            for (_node_id, node_info) in nodes {
                
                let host = node_info["host"].as_str().ok_or_else(|| anyhow!("test"))?;
                let cpu_usage = node_info["os"]["cpu"]["percent"].as_i64().ok_or_else(|| anyhow!("test"))?;
                let jvm_usage = node_info["jvm"]["mem"]["heap_used_percent"].as_i64().ok_or_else(|| anyhow!("test"))?;
                
                let disk_total = node_info["fs"]["total"]["total_in_bytes"].as_i64().ok_or_else(|| anyhow!("test"))?;
                let disk_available = node_info["fs"]["total"]["available_in_bytes"].as_i64().ok_or_else(|| anyhow!("test"))?;
                let disk_usage = ((disk_total - disk_available) as f64 / disk_total as f64) * 100.0;
                
                let metric_info = MetricInfo::new(host.to_string(), jvm_usage, cpu_usage, disk_usage.round() as i64);
                metric_vec.push(metric_info);
            }
            
            for metric in metric_vec {
                let document: Value = serde_json::to_value(&metric)?;
                self.elastic_obj.post_doc(&index_name, document).await?;
            }
        }

        Ok(())
    }

    
    /*
        오늘 날짜 기준 5일전 인덱스는 모두 지워준다.
    */
    async fn delete_cluster_index(&self) -> Result<(), anyhow::Error> {

        let cur_utc_time = get_current_utc_naivedate();
        
        let mut delete_index_list: Vec<String> = Vec::new();
        let res = self.elastic_obj.get_index_belong_pattern("metric_info*").await?;
        
        if let Some(index_obj) = res.as_array() {
            
            for index in index_obj {
                let index_name = index["index"].as_str().ok_or_else(|| anyhow!("test"))?;
                let word_split: Vec<&str> = index_name.split('_').collect();
                let date = word_split.get(2).ok_or_else(|| anyhow!("test"))?;
                
                let parsed_date = NaiveDate::parse_from_str(date, "%Y%m%d")?;
                let five_days_ago = cur_utc_time - chrono::Duration::days(5);

                if parsed_date <= five_days_ago {
                    delete_index_list.push(index_name.to_string());
                }
            }
        }
        
        // 날짜가 5일이 지난 인덱스는 제거.
        for delete_index in delete_index_list {
            self.elastic_obj.delete_index(&delete_index).await?;
            info!("{} index removed", delete_index);  
        }                

        Ok(())
    }
} 