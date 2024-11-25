use crate::common::*;


use crate::utils_modules::time_utils::*;
use crate::utils_modules::json_utils::*;
use crate::utils_modules::calculate_utils::*;

use crate::model::MessageFormatter::*;
use crate::model::Indicies::*;
use crate::model::MetricInfo::*;

use crate::service::tele_bot_service::*;

use crate::repository::es_repository::*;
use crate::repository::smtp_repository::*;

#[async_trait]
pub trait MetricService {
    async fn get_cluster_node_check(&self) -> Result<bool, anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(&self, cluster_status: &str) -> Result<(), anyhow::Error>;
    //async fn get_cluster_pending_tasks(&self) -> Result<(), anyhow::Error>;
    
    async fn get_nodes_stats_handle(&self, metric_vec: &mut Vec<MetricInfo>, cur_utc_time_str: &str) -> Result<(), anyhow::Error>;
    async fn get_cat_shards_handle(&self, metric_vec: &mut Vec<MetricInfo>) -> Result<(), anyhow::Error>;
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error>;
    
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(&self, msg_fmt: &T) -> Result<(), anyhow::Error>;
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
    

    #[doc = "문제가 발생했을 때 알람을 보내주는 함수."]
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(&self, msg_fmt: &T) -> Result<(), anyhow::Error> {
        
        /* Telegram 메시지 Send */
        let tele_service = get_telegram_service();
        let telegram_format = msg_fmt.get_telegram_format();
        tele_service.bot_send(telegram_format.as_str()).await?; 

        /* Email 전송 */
        let smtp_repo = get_smtp_repo();
        let email_format: HtmlContents = msg_fmt.get_email_format();
        smtp_repo.send_message_to_receivers(&email_format).await?;

        
        Ok(())
    }
    
    
    #[doc="Elasticsearch 클러스터 내의 각 노드의 상태를 체크해주는 함수"]
    async fn get_cluster_node_check(&self) -> Result<bool, anyhow::Error> {
        
        /* Vec<(host 주소, 연결 유무)> */
        let conn_stats: Vec<(String, bool)> = self.elastic_obj.get_node_conn_check().await;
        
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
            
            let msg_fmt = MessageFormatterNode::new(
                self.elastic_obj.get_cluster_name(), 
                conn_fail_hosts, 
                String::from("Elasticsearch Connection Failed"), 
                String::from("The connection of these hosts has been LOST."));
            
            self.send_alarm_infos(&msg_fmt).await?;
            
            info!("{:?}", &msg_fmt);

            return Ok(false);
        }
        
        Ok(true)
    }
    

    
    #[doc="Cluster 의 상태를 반환해주는 함수 -> green, yellow, red"]
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error> {
        
        /* 클러스터 상태 체크 */ 
        let cluster_status_json: Value = self.elastic_obj.get_health_info().await?;
        
        let cluster_status = cluster_status_json.get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("[Parsing Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?
            .to_uppercase();
        
        Ok(cluster_status)
    }
    
    
    #[doc="불안정한 인덱스들을 추출하는 함수"]
    async fn get_cluster_unstable_index_infos(&self, cluster_status: &str) -> Result<(), anyhow::Error> {

        let cluster_stat_resp = self.elastic_obj.get_indices_info().await?;
        let unstable_indicies = cluster_stat_resp.trim().lines();
        
        /* 인덱스 상태 확인 및 벡터 생성 */ 
        let mut err_index_detail: Vec<Indicies> = Vec::new();

        for index in unstable_indicies {
            let stats: Vec<&str> = index.split_whitespace().collect();
            
            /* for prod */ 
            match stats.as_slice() {
                [health, status, index, ..] if *health != "green" || *status != "open" => {
                    err_index_detail.push(Indicies::new(index.to_string(), health.to_string().to_uppercase(), status.to_string().to_uppercase()));
                }
                [..] => continue, /* 상태가 안정적인 경우는 무시 */ 
            }
                      
            /* for test */ 
            // match stats.as_slice() {
            //     [health, status, index, ..] if *health == "green" || *status == "open" => {
            //         err_index_detail.push(Indicies::new(index.to_string(), health.to_string().to_uppercase(), status.to_string().to_uppercase()));

            //     }
            //     [..] => continue, /* 상태가 안정적인 경우는 무시 */ 
            // }
        }
        
        err_index_detail.sort_by(|a, b| a.index_name.cmp(&b.index_name));

        
        let msg_fmt = MessageFormatterIndex::new(
            self.elastic_obj.get_cluster_name(), 
            self.elastic_obj.get_cluster_all_host_infos(), 
            String::from(format!("Elasticsearch Cluster health is [{}]", cluster_status)),
            err_index_detail
        );
        
        self.send_alarm_infos(&msg_fmt).await?;  
        
        info!("{:?}", msg_fmt);
        
        Ok(())
    }

    
    // #[doc="중단된 작업 리스트를 확인해주는 함수 (deprecated)"]
    // async fn get_cluster_pending_tasks(&self) -> Result<(), anyhow::Error> {
        
    //     println!("??");

    //     let pending_task = self.elastic_obj
    //         .get_pendging_tasks()
    //         .await?;

    //     /* tasks 필드가 배열로 존재하는지 확인 */ 
    //     let tasks = match pending_task["tasks"].as_array() {
    //         Some(tasks) if !tasks.is_empty() => tasks,
    //         _ => return Ok(()),  /* tasks가 없거나, 빈 배열이면 작업 종료 */ 
    //     };
        
    //     /* 모든 pending task를 문자열로 변환하여 한 번에 처리 */ 
    //     let task_details = tasks
    //         .iter()
    //         .map(|task| task.to_string())
    //         .collect::<Vec<String>>()
    //         .join("\n");
        
    //     println!("{:?}", task_details);
        
    //     /* 메세지 포맷 생성 */ 
    //     // let msg_fmt = MessageFormatter::new(
    //     //     self.elastic_obj.get_cluster_name(), 
    //     //     String::new(),  // 빈 문자열
    //     //     "Elasticsearch pending task issue".to_string(),
    //     //     task_details
    //     // );
        
    //     /* 메세지 전송 */ 
    //     // let send_msg = msg_fmt.transfer_msg();
    //     // let tele_service = get_telegram_service();
    //     // tele_service.bot_send(send_msg.as_str()).await?;  
        
    //     // info!("{:?}", msg_fmt);

    //     Ok(())
    // }


    #[doc = "GET /_nodes/stats 정보들을 핸들링 해주는 함수"]
    /// # Arguments
    /// * `metric_vec`          - Elasticsearch 수집 대상 지표 리스트
    /// * `cur_utc_time_str`    - 현재시간 (문자열) 
    /// 
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn get_nodes_stats_handle(&self, metric_vec: &mut Vec<MetricInfo>, cur_utc_time_str: &str) -> Result<(), anyhow::Error> {

        let query_feilds = ["host", "fs", "jvm", "indices", "os", "http"];

        /* GET /_nodes/stats */ 
        let get_nodes_stats: Value = self
            .elastic_obj
            .get_node_stats(&query_feilds)
            .await?;
        
        if let Some(nodes) = get_nodes_stats["nodes"].as_object() { 
            
            for (_node_id, node_info) in nodes {    
                
                let host: String = get_value_by_path(node_info, "host")?;
                let cpu_usage: i64 = get_value_by_path(node_info, "os.cpu.percent")?;
                let jvm_usage: i64 = get_value_by_path(node_info, "jvm.mem.heap_used_percent")?;
                
                let disk_total: i64 = get_value_by_path(node_info, "fs.total.total_in_bytes")?;
                let disk_available: i64 = get_value_by_path(node_info, "fs.total.available_in_bytes")?;
                let disk_usage = get_percentage_round_conversion(disk_total - disk_available, disk_total, 2)?;
                
                let jvm_young_usage: i64 = get_value_by_path(node_info, "jvm.mem.pools.young.used_in_bytes")?;
                let jvm_old_usage: i64 = get_value_by_path(node_info, "jvm.mem.pools.old.used_in_bytes")?;
                let jvm_survivor_usage: i64 = get_value_by_path(node_info, "jvm.mem.pools.survivor.used_in_bytes")?;
                
                let query_cache_total_cnt: i64 = get_value_by_path(node_info, "indices.query_cache.total_count")?;
                let query_cache_hit_cnt: i64 = get_value_by_path(node_info, "indices.query_cache.hit_count")?;
                let query_cache_hit = get_percentage_round_conversion(query_cache_hit_cnt, query_cache_total_cnt, 2)?;
                let cache_memory_size: i64 = get_value_by_path(node_info, "indices.query_cache.memory_size_in_bytes")?;
                
                let os_swap_total_in_bytes: i64 = get_value_by_path(node_info, "os.swap.total_in_bytes")?;
                let os_swap_used_in_bytes: i64 = get_value_by_path(node_info, "os.swap.used_in_bytes")?;
                let os_swap_usage = get_percentage_round_conversion(os_swap_used_in_bytes, os_swap_total_in_bytes, 2)?;
                
                let http_current_open: i64 = get_value_by_path(node_info, "http.current_open")?;
                
                let indexing_total: i64 = get_value_by_path(node_info, "indices.indexing.index_total")?;
                let index_time_in_millis: i64 = get_value_by_path(node_info, "indices.indexing.index_time_in_millis")?;
                let indexing_latency = get_decimal_round_conversion(index_time_in_millis as f64 / indexing_total as f64, 5)?;


                let query_total: i64 = get_value_by_path(node_info, "indices.search.query_total")?;
                let query_time_in_millis: i64 = get_value_by_path(node_info, "indices.search.query_time_in_millis")?;
                let query_latency = query_time_in_millis as f64 / query_total as f64;
                

                let fetch_total: i64 = get_value_by_path(node_info, "indices.search.fetch_total")?;
                let fetch_time_in_millis: i64 = get_value_by_path(node_info, "indices.search.fetch_time_in_millis")?;
                let fetch_latency = fetch_time_in_millis as f64 / fetch_total as f64;


                let metric_info = MetricInfo::new(
                    cur_utc_time_str.to_string(), 
                    host.to_string(), 
                    jvm_usage, 
                    cpu_usage, 
                    disk_usage.round() as i64,
                    jvm_young_usage,
                    jvm_old_usage,
                    jvm_survivor_usage,
                    query_cache_hit,
                    cache_memory_size,
                    os_swap_total_in_bytes,
                    os_swap_usage,
                    http_current_open,
                    0,
                    indexing_latency,
                    query_latency,
                    fetch_latency
                );
                
                metric_vec.push(metric_info);
            }

        }
        
        Ok(())
    }


    #[doc = "GET /_cat/shards 정보들을 핸들링 해주는 함수"]
    async fn get_cat_shards_handle(&self, metric_vec: &mut Vec<MetricInfo>) -> Result<(), anyhow::Error> {

        let query_feilds = ["ip"];

        /* GET /_nodes/stats */ 
        let get_cat_shards: String = self
            .elastic_obj
            .get_cat_shards(&query_feilds)
            .await?;

        let mut host_map: HashMap<String, i64> = HashMap::new();

        let parsed_data: Vec<&str> = get_cat_shards.split('\n').filter(|s| !s.is_empty()).collect();
        
        for ip_host in parsed_data {

            host_map.entry(ip_host.to_string())
                .and_modify(|value| *value += 1)
                .or_insert(1);  
            
        }
        
        for metric_info in metric_vec {
            
            let host_ip = metric_info.host().clone();
            let shard_cnt = host_map.entry(host_ip).or_insert(0);
            
            metric_info.node_shard_cnt = shard_cnt.clone();
        }
        

        Ok(())
    }
    

    #[doc="각 cluster node 들의 정보를 elasticsearch 에 적재하는 함수"]
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error> {

        /* 지표를 저장해줄 인스턴스 벡터. */ 
        let mut metric_vec: Vec<MetricInfo> = Vec::new();

        let cur_utc_time = get_currnet_utc_naivedatetime();
        let cur_utc_time_str = get_str_from_naivedatetime(cur_utc_time, "%Y-%m-%dT%H:%M:%SZ")?;
        
        /* 날짜 기준으로 인덱스 이름 맵핑 */ 
        let index_pattern = self.elastic_obj.get_cluster_index_pattern();
        let index_name = format!("{}{}", index_pattern, get_str_from_naivedatetime(cur_utc_time, "%Y%m%d")?);

        /* 1. GET /_nodes/stats */ 
        self.get_nodes_stats_handle(&mut metric_vec, &cur_utc_time_str).await?;
        
        /* 2. GET /_cat/shards */ 
        self.get_cat_shards_handle(&mut metric_vec).await?;
        
        
        for metric in metric_vec {
            let document: Value = serde_json::to_value(&metric)?;
            self.elastic_obj.post_doc(&index_name, document).await?;
        }


        Ok(())
    }
} 