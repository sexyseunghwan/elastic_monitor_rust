use crate::common::*;

use crate::utils_modules::calculate_utils::*;
use crate::utils_modules::io_utils::*;
use crate::utils_modules::json_utils::*;
use crate::utils_modules::time_utils::*;

use crate::model::config::*;
use crate::model::index_config::*;
use crate::model::index_info::*;
use crate::model::index_metric_info::*;
use crate::model::indicies::*;
use crate::model::message_formatter_dto::message_formatter::*;
use crate::model::message_formatter_dto::message_formatter_index::*;
use crate::model::message_formatter_dto::message_formatter_node::*;
use crate::model::message_formatter_dto::message_formatter_urgent::*;
use crate::model::thread_pool_stat::*;
use crate::model::urgent_dto::urgent_config::*;
use crate::model::urgent_dto::urgent_info::*;
use crate::model::use_case_config::*;
use crate::model::monitoring::{breaker_info::*, metric_info::*, segment_info::*};

use crate::repository::smtp_repository::*;
use crate::repository::tele_bot_repository::*;

use crate::env_configuration::env_config::*;

use crate::traits::metric_service_trait::*;
use crate::traits::es_repository_trait::*;
use crate::traits::smtp_repository_trait::*;

//
#[derive(Clone, Debug)]
pub struct MetricServicePub<R: EsRepository> {
    elastic_obj: R,
}

impl<R: EsRepository> MetricServicePub<R> {
    pub fn new(elastic_obj: R) -> Self {
        let metric_service: MetricServicePub<R> = MetricServicePub { elastic_obj };
        metric_service
    }
}

/* private function 선언부 */
impl<R: EsRepository + Sync + Send> MetricServicePub<R> {
    
    #[doc = "인덱스 뒤에 금일 날짜를 추가해주는 함수"]
    /// # Arguments
    /// * `index_name` - 인덱스 이름
    /// * `dt` - 날짜 타입
    ///
    /// # Returns
    /// * Result<String, anyhow::Error>
    fn get_today_index_name(&self, index_name: &str, dt: NaiveDateTime) -> Result<String, anyhow::Error> {
        let date: String = get_str_from_naivedatetime(dt, "%Y%m%d")?;
        Ok(format!(
            "{}{}",
            index_name,
            date
        ))
    }

    #[doc = "breaker 모니터링 정보를 수집해주기 위한 함수"]
    /// # Arguments
    /// * `node_info` - 모니터링 정보들
    /// * `name` - 상세 모니터링 필드 이름
    ///
    /// # Returns
    /// * Result<BreakerInfo, anyhow::Error>
    fn get_breaker_info(&self, node_info: &Value, name: &str) -> Result<BreakerInfo, anyhow::Error> {
        let prefix: String = format!("breakers.{}", name);

        let limit_size_in_bytes: u64 = get_value_by_path(node_info, &format!("{}.limit_size_in_bytes", prefix))?;
        let estimated_size_in_bytes: u64 = get_value_by_path(node_info, &format!("{}.estimated_size_in_bytes", prefix))?;
        let tripped: u64 = get_value_by_path(node_info, &format!("{}.tripped", prefix))?;
        Ok(BreakerInfo::new(limit_size_in_bytes, estimated_size_in_bytes, tripped))
    }

    #[doc = "segment 모니터링 정보를 수집해주기 위한 함수"]
    /// # Arguments
    /// * `node_info` - 모니터링 정보들
    ///
    /// # Returns
    /// * Result<SegmentInfo, anyhow::Error>
    fn get_segment_info(&self, node_info: &Value) -> Result<SegmentInfo, anyhow::Error> {

        let segment_count: u64 = get_value_by_path(node_info, "indices.segments.count")?;
        let segment_memory_in_byte: u64 = get_value_by_path(node_info, "indices.segments.memory_in_bytes")?;
        let segment_terms_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.terms_memory_in_bytes")?;
        let segment_stored_fields_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.stored_fields_memory_in_bytes")?;
        let segment_term_vectors_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.term_vectors_memory_in_bytes")?;
        let segment_norms_memory_in_byte: u64 = get_value_by_path(node_info, "indices.segments.norms_memory_in_bytes")?;
        let segment_points_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.points_memory_in_bytes")?;
        let segment_doc_values_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.doc_values_memory_in_bytes")?;
        let segment_index_writer_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.index_writer_memory_in_bytes")?;
        let segment_version_map_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.version_map_memory_in_bytes")?;
        let segment_fixed_bit_set_memory_in_bytes: u64 = get_value_by_path(node_info, "indices.segments.fixed_bit_set_memory_in_bytes")?;

        Ok(
            SegmentInfo::new(
                segment_count, segment_memory_in_byte, segment_terms_memory_in_bytes, segment_stored_fields_memory_in_bytes, segment_term_vectors_memory_in_bytes,
                segment_norms_memory_in_byte, segment_points_memory_in_bytes, segment_doc_values_memory_in_bytes, segment_index_writer_memory_in_bytes,
                segment_version_map_memory_in_bytes, segment_fixed_bit_set_memory_in_bytes
            )
        )
    }



} 


#[async_trait]
impl<R: EsRepository + Sync + Send> MetricService for MetricServicePub<R> {
    
    #[doc = "문제가 발생했을 때 알람을 보내주는 함수."]
    /// # Arguments
    /// * `msg_fmt` - 메시지 포멧터 트레잇
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(
        &self,
        msg_fmt: &T,
    ) -> Result<(), anyhow::Error> {
        /* Telegram 메시지 Send */
        let tele_service: Arc<TelebotRepositoryPub> = get_telegram_repo();
        let telegram_format: String = msg_fmt.get_telegram_format();
        tele_service.bot_send(telegram_format.as_str()).await?;

        /* Email 전송 */
        let smtp_repo: Arc<SmtpRepositoryPub> = get_smtp_repo();
        let email_format: HtmlContents = msg_fmt.get_email_format();
        smtp_repo.send_message_to_receivers(&email_format).await?;

        Ok(())
    }

    #[doc = "Elasticsearch 클러스터 내의 각 노드의 상태를 체크해주는 함수"]
    async fn get_cluster_node_check(&self) -> Result<(), anyhow::Error> {
        /* Vec<(host 주소, 연결 유무)> */
        let conn_stats: Vec<(String, bool)> = self.elastic_obj.get_node_conn_check().await;

        let conn_fail_hosts: Vec<String> = conn_stats
            .into_iter()
            .filter_map(
                |(es_host, is_success)| {
                    if !is_success {
                        Some(es_host)
                    } else {
                        None
                    }
                },
            )
            .collect();

        if !conn_fail_hosts.is_empty() {
            let msg_fmt: MessageFormatterNode = MessageFormatterNode::new(
                self.elastic_obj.get_cluster_name(),
                conn_fail_hosts,
                String::from("Elasticsearch Connection Failed"),
                String::from("The connection of these hosts has been LOST."),
            );

            self.send_alarm_infos(&msg_fmt).await?;
        }

        Ok(())
    }

    #[doc = "Cluster 의 상태를 반환해주는 함수 -> green, yellow, red"]
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error> {
        /* 클러스터 상태 체크 */
        let cluster_status_json: Value = self.elastic_obj.get_health_info().await?;

        let cluster_status: String = cluster_status_json.get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("[Parsing Error][get_cluster_state()] 'status' field is missing in cluster_status_json"))?
            .to_uppercase();

        Ok(cluster_status)
    }

    #[doc = "Elasticsearch Cluster Health 가 불안정한 경우 - 불안정한 인덱스들을 추출하는 함수"]
    async fn get_cluster_unstable_index_infos(
        &self,
        cluster_status: &str,
    ) -> Result<(), anyhow::Error> {
        let cluster_stat_resp: String = self.elastic_obj.get_indices_info().await?;
        let unstable_indicies: Lines<'_> = cluster_stat_resp.trim().lines();

        /* 현재 프로그램 실행환경 구분 */
        let use_case_binding: Arc<UseCaseConfig> = get_usecase_config_info();
        let use_case: &str = use_case_binding.use_case().as_str();

        /* 인덱스 상태 확인 및 벡터 생성 */
        let mut err_index_detail: Vec<Indicies> = Vec::new();

        for index in unstable_indicies {
            let stats: Vec<&str> = index.split_whitespace().collect();

            let (health, status, index) = match stats.as_slice() {
                [health, status, index, ..] => (health, status, index),
                _ => continue,
            };

            /* 개발환경, 운영환경 코드 구분 */
            // let is_unstable: bool = match use_case {
            //     "dev" => *health == "red" || *status == "open",
            //     "prod" => *health != "green" || *status != "open",
            //     _ => false,
            // };

            let is_unstable: bool = match use_case {
                "dev" => *health != "green" || *status != "open",
                "prod" => *health != "green" || *status != "open",
                _ => false,
            };

            if is_unstable {
                err_index_detail.push(Indicies::new(
                    index.to_string(),
                    health.to_uppercase(),
                    status.to_uppercase(),
                ));
            }
        }

        err_index_detail.sort_by(|a, b| a.index_name.cmp(&b.index_name));

        let msg_fmt: MessageFormatterIndex = MessageFormatterIndex::new(
            self.elastic_obj.get_cluster_name(),
            self.elastic_obj.get_cluster_all_host_infos(), 
            String::from(format!(
                "Elasticsearch Cluster health is [{}]",
                cluster_status
            )),
            err_index_detail,
        );

        self.send_alarm_infos(&msg_fmt).await?;

        Ok(())
    }

    #[doc = "GET /_nodes/stats 정보들을 핸들링 해주는 함수"]
    /// # Arguments
    /// * `metric_vec`          - Elasticsearch 수집 대상 지표 리스트
    /// * `cur_utc_time_str`    - 현재시간 (문자열)
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn get_nodes_stats_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
        cur_utc_time_str: &str,
    ) -> Result<(), anyhow::Error> {
        let query_fields: [&str; 6] = ["fs", "jvm", "indices", "os", "http", "breaker"];

        /* GET /_nodes/stats */
        let get_nodes_stats: Value = self.elastic_obj.get_node_stats(&query_fields).await?;

        if let Some(nodes) = get_nodes_stats["nodes"].as_object() {
            for (_node_id, node_info) in nodes {
                let host: String = get_value_by_path(node_info, "host")?;
                let name: String = get_value_by_path(node_info, "name")?;

                let cpu_usage: i64 = get_value_by_path(node_info, "os.cpu.percent")?;
                let jvm_usage: i64 = get_value_by_path(node_info, "jvm.mem.heap_used_percent")?;

                let disk_total: i64 = get_value_by_path(node_info, "fs.total.total_in_bytes")?;
                let disk_available: i64 =
                    get_value_by_path(node_info, "fs.total.available_in_bytes")?;

                let disk_usage: f64 =
                    get_percentage_round_conversion(disk_total - disk_available, disk_total, 2)?;

                let jvm_young_usage: i64 =
                    get_value_by_path(node_info, "jvm.mem.pools.young.used_in_bytes")?;
                let jvm_old_usage: i64 =
                    get_value_by_path(node_info, "jvm.mem.pools.old.used_in_bytes")?;
                let jvm_survivor_usage: i64 =
                    get_value_by_path(node_info, "jvm.mem.pools.survivor.used_in_bytes")?;

                let jvm_buffer_pool_mapped_count: u64 =
                    get_value_by_path(node_info, "jvm.buffer_pools.mapped.count")?;
                let jvm_buffer_pool_mapped_use_byte: u64 =
                    get_value_by_path(node_info, "jvm.buffer_pools.mapped.used_in_bytes")?;
                let jvm_buffer_pool_mapped_total_byte: u64 = get_value_by_path(
                    node_info,
                    "jvm.buffer_pools.mapped.total_capacity_in_bytes",
                )?;

                let jvm_buffer_pool_direct_count: u64 =
                    get_value_by_path(node_info, "jvm.buffer_pools.direct.count")?;
                let jvm_buffer_pool_direct_use_byte: u64 =
                    get_value_by_path(node_info, "jvm.buffer_pools.direct.used_in_bytes")?;
                let jvm_buffer_pool_direct_total_byte: u64 = get_value_by_path(
                    node_info,
                    "jvm.buffer_pools.direct.total_capacity_in_bytes",
                )?;

                let query_cache_total_cnt: i64 =
                    get_value_by_path(node_info, "indices.query_cache.total_count")?;
                let query_cache_hit_cnt: i64 =
                    get_value_by_path(node_info, "indices.query_cache.hit_count")?;
                let query_cache_hit: f64 =
                    get_percentage_round_conversion(query_cache_hit_cnt, query_cache_total_cnt, 2)?;

                let cache_memory_size: i64 =
                    get_value_by_path(node_info, "indices.query_cache.memory_size_in_bytes")?;

                let os_swap_total_in_bytes: i64 =
                    get_value_by_path(node_info, "os.swap.total_in_bytes")?;
                let os_swap_used_in_bytes: i64 =
                    get_value_by_path(node_info, "os.swap.used_in_bytes")?;

                let os_swap_usage: f64 = get_percentage_round_conversion(
                    os_swap_used_in_bytes,
                    os_swap_total_in_bytes,
                    2,
                )?;

                let http_current_open: i64 = get_value_by_path(node_info, "http.current_open")?;

                let indexing_total: i64 =
                    get_value_by_path(node_info, "indices.indexing.index_total")?;
                let index_time_in_millis: i64 =
                    get_value_by_path(node_info, "indices.indexing.index_time_in_millis")?;
                let indexing_latency: f64 = get_decimal_round_conversion(
                    index_time_in_millis as f64 / indexing_total as f64,
                    5,
                )?;

                let query_total: i64 = get_value_by_path(node_info, "indices.search.query_total")?;
                let query_time_in_millis: i64 =
                    get_value_by_path(node_info, "indices.search.query_time_in_millis")?;
                let query_latency: f64 = query_time_in_millis as f64 / query_total as f64;

                let fetch_total: i64 = get_value_by_path(node_info, "indices.search.fetch_total")?;
                let fetch_time_in_millis: i64 =
                    get_value_by_path(node_info, "indices.search.fetch_time_in_millis")?;
                let fetch_latency: f64 = fetch_time_in_millis as f64 / fetch_total as f64;

                let translog_operation: i64 =
                    get_value_by_path(node_info, "indices.translog.operations")?;
                let translog_operation_size: i64 =
                    get_value_by_path(node_info, "indices.translog.size_in_bytes")?;
                let translog_uncommited_operation: i64 =
                    get_value_by_path(node_info, "indices.translog.uncommitted_operations")?;
                let translog_uncommited_operation_size: i64 =
                    get_value_by_path(node_info, "indices.translog.uncommitted_size_in_bytes")?;

                let flush_total: i64 = get_value_by_path(node_info, "indices.flush.total")?;

                let refresh_total: i64 = get_value_by_path(node_info, "indices.refresh.total")?;
                let refresh_listener: i64 =
                    get_value_by_path(node_info, "indices.refresh.listeners")?;

                /* off-heap 메모리 관리를 위한 모니터링 코드 추가 */
                let segment_infos: SegmentInfo = self.get_segment_info(node_info)?;
                

                /* breaker 지표 관련 */
                let breaker_request: BreakerInfo = self.get_breaker_info(node_info, "request")?;
                let breaker_fielddata: BreakerInfo = self.get_breaker_info(node_info, "fielddata")?;
                let breaker_inflight_requests: BreakerInfo = self.get_breaker_info(node_info, "inflight_requests")?;
                let breaker_parent: BreakerInfo = self.get_breaker_info(node_info, "parent")?;                

                /* 이후에 값이 들어가야 하는 필드인 경우에는 지금 해당 소스에서 0으로 초기화 한 후에 데이터를 넣어준다. */
                let metric_info: MetricInfo = MetricInfoBuilder::default()
                    .timestamp(cur_utc_time_str.to_string())
                    .host(host.to_string())
                    .name(name)
                    .jvm_usage(jvm_usage)
                    .cpu_usage(cpu_usage)
                    .disk_usage(disk_usage.round() as i64)
                    .jvm_young_usage_byte(jvm_young_usage)
                    .jvm_old_usage_byte(jvm_old_usage)
                    .jvm_survivor_usage_byte(jvm_survivor_usage)
                    .jvm_buffer_pool_mapped_count(jvm_buffer_pool_mapped_count)
                    .jvm_buffer_pool_mapped_use_byte(jvm_buffer_pool_mapped_use_byte)
                    .jvm_buffer_pool_mapped_total_byte(jvm_buffer_pool_mapped_total_byte)
                    .jvm_buffer_pool_direct_count(jvm_buffer_pool_direct_count)
                    .jvm_buffer_pool_direct_use_byte(jvm_buffer_pool_direct_use_byte)
                    .jvm_buffer_pool_direct_total_byte(jvm_buffer_pool_direct_total_byte)
                    .query_cache_hit(query_cache_hit)
                    .cache_memory_size(cache_memory_size)
                    .os_swap_total_in_bytes(os_swap_total_in_bytes)
                    .os_swap_usage(os_swap_usage)
                    .http_current_open(http_current_open)
                    .node_shard_cnt(0)
                    .indexing_latency(indexing_latency)
                    .query_latency(query_latency)
                    .fetch_latency(fetch_latency)
                    .translog_operation(translog_operation)
                    .translog_operation_size(translog_operation_size)
                    .translog_uncommited_operation(translog_uncommited_operation)
                    .translog_uncommited_operation_size(translog_uncommited_operation_size)
                    .flush_total(flush_total)
                    .refresh_total(refresh_total)
                    .refresh_listener(refresh_listener)
                    .search_active_thread(0)
                    .search_thread_queue(0)
                    .search_rejected_thread(0)
                    .write_active_thread(0)
                    .write_thread_queue(0)
                    .write_rejected_thread(0)
                    .bulk_active_thread(0)
                    .bulk_thread_queue(0)
                    .bulk_rejected_thread(0)
                    .get_active_thread(0)
                    .get_thread_queue(0)
                    .get_rejected_thread(0)
                    .menagement_active_thread(0)
                    .menagement_thread_queue(0)
                    .menagement_rejected_thread(0)
                    .generic_active_thread(0)
                    .generic_thread_queue(0)
                    .generic_rejected_thread(0)
                    .segment_infos(segment_infos)
                    .breaker_request(breaker_request)
                    .breaker_fielddata(breaker_fielddata)
                    .breaker_inflight_requests(breaker_inflight_requests)
                    .breaker_parent(breaker_parent)
                    .build()?;

                metric_vec.push(metric_info);
            }
        }

        Ok(())
    }

    #[doc = "GET /{index_name}/stats 정보들을 핸들링 해주는 함수"]
    /// # Arguments
    /// * `index_name`          - 인덱스 이름
    /// * `cur_utc_time_str`    - 현재시간 (문자열)
    ///
    /// # Returns
    /// * Result<IndexMetricInfo, anyhow::Error>
    async fn get_index_stats_handle(
        &self,
        index_name: &str,
        cur_utc_time_str: &str,
    ) -> Result<IndexMetricInfo, anyhow::Error> {
        let get_index_stats: Value = self.elastic_obj.get_specific_index_info(index_name).await?;

        if let Some(total_stats) = get_index_stats.get("_all") {
            let translog_operation: i64 =
                get_value_by_path(total_stats, "total.translog.operations")?;
            let translog_operation_size: i64 =
                get_value_by_path(total_stats, "total.translog.size_in_bytes")?;
            let translog_uncommited_operation: i64 =
                get_value_by_path(total_stats, "total.translog.uncommitted_operations")?;
            let translog_uncommited_operation_size: i64 =
                get_value_by_path(total_stats, "total.translog.uncommitted_size_in_bytes")?;

            let flush_total: i64 = get_value_by_path(total_stats, "total.flush.total")?;

            let refresh_total: i64 = get_value_by_path(total_stats, "total.refresh.total")?;
            let refresh_listener: i64 = get_value_by_path(total_stats, "total.refresh.listeners")?;

            let index_metric_info: IndexMetricInfo = IndexMetricInfo::new(
                cur_utc_time_str.to_string(),
                index_name.to_string(),
                translog_operation,
                translog_operation_size,
                translog_uncommited_operation,
                translog_uncommited_operation_size,
                flush_total,
                refresh_total,
                refresh_listener,
            );

            Ok(index_metric_info)
        } else {
            Err(anyhow!("[Error][MetricService->get_index_stats_handle] No _all.total section in stats response"))
        }
    }

    #[doc = "GET /_cat/shards 정보들을 핸들링 해주는 함수"]
    /// # Arguments
    /// * `metric_vec`          - 모니터링 지표 리스트
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn get_cat_shards_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error> {
        let query_feilds: [&str; 1] = ["ip"];

        /* GET /_nodes/stats */
        let get_cat_shards: String = self.elastic_obj.get_cat_shards(&query_feilds).await?;

        let mut host_map: HashMap<String, i64> = HashMap::new();

        let parsed_data: Vec<&str> = get_cat_shards
            .split('\n')
            .filter(|s| !s.is_empty())
            .collect();

        for ip_host in parsed_data {
            host_map
                .entry(ip_host.to_string())
                .and_modify(|value| *value += 1)
                .or_insert(1);
        }

        for metric_info in metric_vec {
            let host_ip: String = metric_info.host().clone();
            let shard_cnt: &mut i64 = host_map.entry(host_ip).or_insert(0);

            metric_info.node_shard_cnt = shard_cnt.clone();
        }

        Ok(())
    }

    #[doc = "GET /_cat/thread_pool 정보들을 핸들링 해주는 함수"]
    /// # Arguments
    /// * `metric_vec`          - 모니터링 지표 리스트
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn get_cat_thread_pool_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error> {
        let query_feilds: [&str; 6] = ["search", "write", "bulk", "get", "management", "generic"];

        let get_cat_thread_pool: String = self.elastic_obj.get_cat_thread_pool().await?;

        let thread_pool_stats: Vec<ThreadPoolStat> = get_cat_thread_pool
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();

                if parts.len() >= 5 && query_feilds.contains(&parts[1]) {
                    Some(ThreadPoolStat::new(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts[2].parse().unwrap_or(0),
                        parts[3].parse().unwrap_or(0),
                        parts[4].parse().unwrap_or(0),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let mut map: HashMap<String, Vec<ThreadPoolStat>> = HashMap::new();

        for stat in thread_pool_stats {
            map.entry(stat.node_name().clone())
                .or_insert(Vec::new())
                .push(stat);
        }

        for metric in metric_vec {
            let node_name = metric.name();
            let thread_pool_stat: &Vec<ThreadPoolStat> = match map.get(node_name) {
                Some(thread_pool_stat) => thread_pool_stat,
                None => {
                    error!("[Error][MetricService->get_cat_thread_pool_handle] The information corresponding to {} does not exist.", node_name);
                    continue;
                }
            };

            for stat in thread_pool_stat {
                match stat.name().as_str() {
                    "search" => {
                        metric.search_active_thread = *stat.active();
                        metric.search_thread_queue = *stat.queue();
                        metric.search_rejected_thread = *stat.rejected();
                    }
                    "write" => {
                        metric.write_active_thread = *stat.active();
                        metric.write_thread_queue = *stat.queue();
                        metric.write_rejected_thread = *stat.rejected();
                    }
                    "bulk" => {
                        metric.bulk_active_thread = *stat.active();
                        metric.bulk_thread_queue = *stat.queue();
                        metric.bulk_rejected_thread = *stat.rejected();
                    }
                    "get" => {
                        metric.get_active_thread = *stat.active();
                        metric.get_thread_queue = *stat.queue();
                        metric.get_rejected_thread = *stat.rejected();
                    }
                    "management" => {
                        metric.menagement_active_thread = *stat.active();
                        metric.menagement_thread_queue = *stat.queue();
                        metric.menagement_rejected_thread = *stat.rejected();
                    }
                    "generic" => {
                        metric.generic_active_thread = *stat.active();
                        metric.generic_thread_queue = *stat.queue();
                        metric.generic_rejected_thread = *stat.rejected();
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    #[doc = "각 cluster node 들의 정보를 elasticsearch 에 적재하는 함수"]
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error> {
        /* 지표를 저장해줄 인스턴스 벡터. */
        let mut metric_vec: Vec<MetricInfo> = Vec::new();

        let now: NaiveDateTime = get_currnet_utc_naivedatetime();
        let now_str: String = format_datetime(now)?;

        /* 날짜 기준으로 인덱스 이름 맵핑 */
        let index_name: String = self
            .get_today_index_name(self.elastic_obj.get_cluster_index_pattern().as_str(), now)?;

        /* 1. GET /_nodes/stats */
        self.get_nodes_stats_handle(&mut metric_vec, &now_str)
            .await?;

        /* 2. GET /_cat/shards */
        self.get_cat_shards_handle(&mut metric_vec).await?;

        /* 3. GET /_cat/thread_pool */
        self.get_cat_thread_pool_handle(&mut metric_vec).await?;

        for metric in metric_vec {
            let document: Value = serde_json::to_value(&metric)?;
            self.elastic_obj.post_doc(&index_name, document).await?;
        }

        Ok(())
    }

    #[doc = "모니터링 대상이 되는 index의 개별 정보를 elasticsearch 에 적재하는 함수"]
    async fn post_cluster_index_infos(&self) -> Result<(), anyhow::Error> {
        let cluster_name: String = self.elastic_obj.get_cluster_name();
        
        let now: NaiveDateTime = get_currnet_utc_naivedatetime();
        let now_str: String = format_datetime(now)?;

        /* 인덱스 이름 생성 */ 
        let index_name: String = self
            .get_today_index_name(self.elastic_obj.get_cluster_index_monitoring_pattern().as_str(), now)?;

        let monitor_indexies: IndexConfig =
            read_toml_from_file::<IndexConfig>(&ELASTIC_INDEX_INFO_PATH)?;

        let index_vec: Vec<IndexInfo> = monitor_indexies
            .index
            .into_iter()
            .filter(|elem| *elem.cluster_name() == cluster_name)
            .collect();

        for elem in index_vec {
            let index_matric_info: IndexMetricInfo = self
                .get_index_stats_handle(elem.index_name(), &now_str)
                .await?;

            let document: Value = serde_json::to_value(&index_matric_info)?;
            self.elastic_obj.post_doc(&index_name, document).await?;
        }

        Ok(())
    }
    
    #[doc = "긴급한 지표를 모니터링 한 뒤 알람을 보내주는 함수"]
    async fn send_alarm_urgent_infos(&self) -> Result<(), anyhow::Error> {
        let cluster_name: String = self.elastic_obj.get_cluster_name();

        let now: NaiveDateTime = get_currnet_utc_naivedatetime();
        let past: NaiveDateTime = now - chrono::Duration::seconds(20);
        
        let now_str: String = format_datetime(now)?;
        let past_str: String = format_datetime(past)?;

        /* 인덱스 이름 생성 */ 
        let index_name: String = self
            .get_today_index_name(self.elastic_obj.get_cluster_index_urgent_pattern().as_str(),now)?;
        
        /* 긴급 모니터링 구성 로딩 */
        let urgent_configs: UrgentConfigList =
            read_toml_from_file::<UrgentConfigList>(&URGENT_CONFIG_PATH)?;
        
        /* 긴급 지표 모니터링 대상이 되는 노드 아이피 주소 */
        let host_ips: Vec<String> = self.elastic_obj.get_cluster_all_monitor_host_infos()
            .iter()
            .map(|host_info| {
                host_info
                    .split_once(':')
                    .map(|(host, _)| host.to_string()) 
                    .unwrap_or_else(|| host_info.to_string()) 
            })
            .collect();
        
        if host_ips.is_empty() {
            warn!("[Warn][MetricService->send_alarm_urgent_infos] No host IPs found. Skipping query.");
            return Ok(());
        }

        /* elasticsearch shoul query */
        let should_terms: Vec<Value> = host_ips
            .iter()
            .map(|ip| json!({ "term": { "host": ip } }))
            .collect();

        /* 엘라스틱 서치 쿼리를 통해서 최근 20초동안 urgent 지표를 확인해준다. */
        let query: Value = json!({
            "query": {
                "bool": {
                    "must": [
                        {
                            "range": {
                                "timestamp": {
                                    "gte": past_str,
                                    "lte": now_str
                                }
                            }
                        },
                        {
                            "bool": {
                                "should": should_terms,
                                "minimum_should_match": 1
                            }
                        }
                    ]
                }
            }
        });

        let urgent_infos: Vec<UrgentInfo> = self
            .elastic_obj
            .get_search_query::<UrgentInfo>(&query, &index_name)
            .await?;

        if urgent_infos.is_empty() {
            info!("[Info][MetricService->send_alarm_urgent_infos] The `urgent_infos` vector is empty.");
            return Ok(());
        }

        /* 알람 대상 필터링 */ 
        let alarm_targets: Vec<UrgentAlarmInfo> = urgent_infos
            .iter()
            .flat_map(|info| {
                urgent_configs
                    .urgent()
                    .iter()
                    .filter_map(|cfg| {
                        let metric: &String = cfg.metric_name();
                        let limit: f64 = *cfg.limit();

                        match info.get_field_value(metric) {
                            Some(val) if val > limit => Some(UrgentAlarmInfo::new(
                                info.host().to_string(),
                                metric.clone(),
                                val.to_string(),
                            )),
                            Some(_) => None,
                            None => {
                                error!(
                                    "[Error][MetricService->send_alarm_urgent_infos] Missing value for metric '{}'",
                                    metric
                                );
                                None
                            }
                        }
                    })
            })
            .collect();
        
        if alarm_targets.is_empty() {
            info!("[Info][MetricService->send_alarm_urgent_infos] No emergency monitoring indicators exist that require an alarm.");
            return Ok(());
        }

        let msg: MessageFormatterUrgent = MessageFormatterUrgent::new(cluster_name, alarm_targets);
        self.send_alarm_infos(&msg).await?;

        Ok(())
    }
}
