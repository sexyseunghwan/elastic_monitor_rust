use crate::common::*;

use crate::traits::{repository::es_repository_trait::*, service::mon_es_service_trait::*};

use crate::model::elastic_dto::dummy_data::*;
use crate::model::elastic_dto::elastic_source_parser::*;
use crate::model::message_formatter_dto::message_formatter_urgent::*;
use crate::model::monitoring::metric_info::*;
use crate::model::reports::{err_agg_history_bucket::*, err_log_info::*};
use crate::model::search_indicies::*;
use crate::model::urgent_dto::{urgent_config::*, urgent_info::*};

use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::env_configuration::env_config::*;

#[derive(Clone, Debug, new)]
pub struct MonEsServiceImpl<R: EsRepository> {
    elastic_obj: Arc<R>,
}

impl<R: EsRepository> MonEsServiceImpl<R> {
    #[doc = "Common function for asynchronously loading error log lists into Elasticsearh"]
    /// # Arguments
    /// * `err_log_list` - Error log list to load
    /// * `err_log_index` - Error log index pattern
    /// * `now_utc` - Current UTC time
    /// * `mon_es` - Elasticsearch connection
    /// * `caller_name` - Name of the called function(for logging)
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn bulk_post_error_logs(
        &self,
        err_log_list: Vec<Value>,
        err_log_index: &str,
        now_utc: DateTime<Utc>,
        caller_name: &str,
    ) -> anyhow::Result<()> {
        let futures = err_log_list.into_iter().map(|doc| {
            let index_name: String =
                format!("{}{}", err_log_index, convert_date_to_str_ymd(now_utc, Utc));
            let mon_es: Arc<R> = Arc::clone(&self.elastic_obj);
            async move { mon_es.post_doc(&index_name, doc).await }
        });

        let results: Vec<std::result::Result<(), anyhow::Error>> =
            futures::future::join_all(futures).await;

        for (idx, result) in results.into_iter().enumerate() {
            if let Err(e) = result {
                error!(
                    "[MetricServiceImpl->{}] Failed to post error log #{}: {:?}",
                    caller_name, idx, e
                );
            }
        }

        Ok(())
    }

    #[doc = "Function that adds today's date after the index."]
    /// # Arguments
    /// * `index_name` - name of index
    /// * `dt` -
    ///
    /// # Returns
    /// * String
    fn get_today_index_name(&self, index_name: &str, dt: DateTime<Utc>) -> String {
        let today_date_str: String = convert_date_to_str_ymd(dt, Utc);
        let index_name: String = format!("{}{}", index_name, today_date_str);
        index_name
    }

    #[doc = "Function that generated queries related to emergency indicators."]
    /// # Arguments
    /// * `host_ips` - 클러스터 내부 노드 아이피주소
    /// * `past_str` - 쿼리 필터링 시작일
    /// * `now_str`  - 쿼리필터링 종료일
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    fn build_urgent_query(&self, host_ips: &[String], past_str: &str, now_str: &str) -> Value {
        /* elasticsearch shoul query */
        let should_terms: Vec<Value> = host_ips
            .iter()
            .map(|ip| json!({ "term": { "host": ip } }))
            .collect();

        /* 엘라스틱 서치 쿼리를 통해서 최근 20초동안 urgent 지표를 확인해준다. */
        json!({
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
        })
    }
}

#[async_trait]
impl<R> MonEsService for MonEsServiceImpl<R>
where
    R: EsRepository + Sync + Send,
{
    #[doc = ""]
    async fn put_node_conn_err_infos(
        &self,
        cluster_name: &str,
        fail_hosts: &[String],
    ) -> anyhow::Result<()> {
        let now_utc: DateTime<Utc> = Utc::now();

        let err_log_index: String = self
            .elastic_obj
            .get_cluster_index_error_pattern()
            .ok_or_else(|| {
                anyhow!("[MonEsServiceImpl->put_node_conn_err_infos] err_log_index is empty")
            })?;

        let err_log_list: Vec<Value> = fail_hosts
            .iter()
            .filter_map(|host| {
                let err_log_info: ErrorLogInfo = ErrorLogInfo::new(
                    cluster_name.to_string(),
                    host.to_string(),
                    String::from(""),
                    convert_date_to_str_full(now_utc, Utc),
                    "Node connection failure".into(),
                    format!("Connection to node {} cannot be confirmed.", host),
                );

                serde_json::to_value(&err_log_info).ok()
            })
            .collect();

        self.bulk_post_error_logs(
            err_log_list,
            &err_log_index,
            now_utc,
            "put_node_conn_err_infos",
        )
        .await
    }

    #[doc = "Function to logs issues when problems occur with Elasticsearch cluster health"]
    async fn put_cluster_health_unstable_infos(
        &self,
        cluster_name: &str,
        danger_indicies: &[SearchIndicies],
    ) -> anyhow::Result<()> {
        let now_utc: DateTime<Utc> = Utc::now();

        let err_log_index: String = self
            .elastic_obj
            .get_cluster_index_error_pattern()
            .ok_or_else(|| {
                anyhow!(
                    "[MonEsServiceImpl->put_cluster_health_unstable_infos] err_log_index is empty"
                )
            })?;

        let err_log_list: Vec<Value> = danger_indicies
            .iter()
            .filter_map(|index| {
                let err_log_info: ErrorLogInfo = ErrorLogInfo::new(
                    cluster_name.to_string(),
                    String::from(""),
                    index.index_name().to_string(),
                    convert_date_to_str_full(now_utc, Utc),
                    "Cluster status is unstable".into(),
                    format!(
                        "The status if the {} index within cluster {} is {}",
                        index.index_name(),
                        cluster_name,
                        index.health()
                    ),
                );

                serde_json::to_value(&err_log_info).ok()
            })
            .collect();

        self.bulk_post_error_logs(
            err_log_list,
            &err_log_index,
            now_utc,
            "put_cluster_health_unstable_infos",
        )
        .await
    }

    #[doc = "Function that logs which metric is problematic when an emergency metric alert occurs"]
    async fn put_urgent_infos(
        &self,
        cluster_name: &str,
        urgent_infos: &[UrgentAlarmInfo],
    ) -> anyhow::Result<()> {
        let now_utc: DateTime<Utc> = Utc::now();

        let err_log_index: String = self
            .elastic_obj
            .get_cluster_index_error_pattern()
            .ok_or_else(|| {
                anyhow!("[MonEsServiceImpl->put_urgent_infos] err_log_index is empty")
            })?;

        let err_log_list: Vec<Value> = urgent_infos
            .iter()
            .filter_map(|urgent_index| {
                let err_log_info: ErrorLogInfo = ErrorLogInfo::new(
                    cluster_name.to_string(),
                    urgent_index.host().to_string(),
                    String::from(""),
                    convert_date_to_str_full(now_utc, Utc),
                    "Emergency indicator alarm dispatch".into(),
                    format!(
                        "{} metric has exceeded the threshold\nMetric value:{}",
                        urgent_index.metric_name(),
                        urgent_index.metric_value_str()
                    ),
                );

                serde_json::to_value(&err_log_info).ok()
            })
            .collect();

        self.bulk_post_error_logs(
            err_log_list,
            &err_log_index,
            now_utc,
            "put_cluster_health_unstable_infos",
        )
        .await
    }

    #[doc = "Function for loading information from each cluster node into Monitoring Elasticsearch"]
    async fn post_cluster_nodes_infos(&self, metric_infos: Vec<MetricInfo>) -> anyhow::Result<()> {
        /* metric_info_log_ ... */
        let cluster_index_pattern: String = self
            .elastic_obj
            .get_cluster_index_pattern()
            .ok_or_else(|| {
                anyhow!(
                    "[MonEsServiceImpl::post_cluster_nodes_infos] cluster_index_pattern is empty"
                )
            })?;

        let now: DateTime<Utc> = Utc::now();
        let index_name: String = self.get_today_index_name(&cluster_index_pattern, now);

        for metric in metric_infos.into_iter() {
            let document: Value = serde_json::to_value(metric)?;

            if let Err(e) = self.elastic_obj.post_doc(&index_name, document).await {
                error!("[MonEsServiceImpl::post_cluster_nodes_infos] {:?}", e);
            }
        }

        Ok(())
    }

    #[doc = "Function that monitors critical metrics and returns the result."]
    async fn get_alarm_urgent_infos(
        &self,
        host_ips: Vec<String>,
    ) -> anyhow::Result<Vec<UrgentAlarmInfo>> {
        let (now, _past, now_str, past_str) = make_time_range(20);

        let cluster_index_urgent_pattern: String = self
            .elastic_obj
            .get_cluster_index_urgent_pattern()
            .ok_or_else(|| anyhow!("[MonEsServiceImpl::get_alarm_urgent_infos] cluster_index_monitor_pattern is empty"))?;

        /* Generate name of index */
        let index_name: String = self.get_today_index_name(&cluster_index_urgent_pattern, now);

        let urgent_configs: UrgentConfigList =
            read_toml_from_file::<UrgentConfigList>(&URGENT_CONFIG_PATH).map_err(|e| {
                anyhow!(
                    "[MonEsServiceImpl::get_alarm_urgent_infos][urgent_configs] {:?}",
                    e
                )
            })?;

        let query: Value = self.build_urgent_query(&host_ips, &past_str, &now_str);
        let urgent_infos: Vec<UrgentInfo> = self
            .elastic_obj
            .get_search_query::<UrgentInfo>(&query, &index_name)
            .await
            .map_err(|e| {
                anyhow!(
                    "[MonEsServiceImpl::get_alarm_urgent_infos][urgent_infos] {:?}",
                    e
                )
            })?;

        if urgent_infos.is_empty() {
            warn!("[MonEsServiceImpl::get_alarm_urgent_infos] The `urgent_infos` vector is empty.");
        }

        let urgent_alarm_infos: Vec<UrgentAlarmInfo> = urgent_infos
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
                                    "[MonEsServiceImpl::get_alarm_urgent_infos] Missing value for metric '{}'",
                                    metric
                                );
                                None
                            }
                        }
                    })
            })
            .collect();

        Ok(urgent_alarm_infos)
    }

    #[doc = "Function that returns error log information related to node failures in Elasticsearch"]
    async fn get_cluster_err_datas_cnt_from_es(
        &self,
        cluster_name: &str,
        err_title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> anyhow::Result<u64> {
        let err_index_name: String = format!(
            "{}*",
            self.elastic_obj.get_cluster_index_error_pattern()
                .ok_or_else(|| anyhow!("[MonEsServiceImpl::get_cluster_err_datas_cnt_from_es]`Error log index pattern` is not configured"))?
        );

        let search_query: Value = json!({
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                "timestamp": {
                                    "gte": convert_date_to_str_full(start_at, Utc),
                                    "lte": convert_date_to_str_full(end_at, Utc)
                                }
                            }
                        },
                        {
                            "term": {
                                "err_title.keyword": err_title
                            }
                        },
                        {
                            "term": {
                                "cluster_name.keyword": cluster_name
                            }
                        }
                    ]
                }
            }
        });

        let err_count: u64 = self
            .elastic_obj
            .get_count_query(&search_query, &err_index_name)
            .await
            .map_err(|e| anyhow!("[MonEsServiceImpl::get_cluster_err_datas_cnt_from_es] Failed to get error count: {:?}", e))?;

        Ok(err_count)
    }

    #[doc = "Retrieve aggregated error log data from Elasticsearch using date histogram aggregation"]
    async fn get_agg_err_datas_from_es(
        &self,
        cluster_name: &str,
        err_title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        calendar_interval: &str,
    ) -> anyhow::Result<Vec<ErrorAggHistoryBucket>> {
        let err_index: String = self.elastic_obj.get_cluster_index_error_pattern()
            .ok_or_else(|| anyhow!("[MonEsServiceImpl::get_agg_err_datas_from_es]`Error log index pattern` is not configured"))?;

        let err_index_name: String = format!("{}*", err_index);

        let has_data: bool = self.elastic_obj
            .check_index_has_data(&err_index_name)
            .await
            .map_err(|e| anyhow!("[MonEsServiceImpl::get_agg_err_datas_from_es] Failed to check index data: {:?}", e))?;

        /*
            If no data exists, dummy data must be inserted.
            The absence of aggregation results is considered an error in all cases.
        */
        if !has_data {
            let dummy_index_name: String = format!("{}19750101", err_index);
            let dummy: DummyData = DummyData::new(String::from("dummy"));
            let dummy_json: Value = serde_json::to_value(dummy).map_err(|e| {
                anyhow!(
                    "[MonEsServiceImpl::get_agg_err_datas_from_es] Failed convert dummy_json: {:?}",
                    e
                )
            })?;

            match self
                .elastic_obj
                .post_doc(&dummy_index_name, dummy_json)
                .await
            {
                Ok(_) => {
                    info!("[MonEsServiceImpl::get_agg_err_datas_from_es] Dummy data generation complete.: {}", dummy_index_name);
                }
                Err(e) => {
                    error!("[MonEsServiceImpl::get_agg_err_datas_from_es] Failed post the `dummy_index` data.: {:?}", e);
                    return Ok(Vec::new());
                }
            }
        }

        let search_query: Value = json!({
            "query": {
                "bool": {
                    "filter": [
                        {
                            "range": {
                                "timestamp": {
                                    "gte": convert_date_to_str_full(start_at, Utc),
                                    "lte": convert_date_to_str_full(end_at, Utc)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          }
                            }
                        },
                        {
                            "term": {
                                "cluster_name.keyword": cluster_name
                            }
                        },
                        {
                            "term": {
                                "err_title.keyword": err_title
                            }
                        }
                    ]
                }
            },
            "aggs": {
                "logs_per_time": {
                    "date_histogram": {
                        "field": "timestamp",
                        "calendar_interval": calendar_interval,
                        "min_doc_count": 0,
                        "extended_bounds": {
                            "min": convert_date_to_str_full(start_at, Utc),
                            "max": convert_date_to_str_full(end_at, Utc)
                        }
                    }
                }
            },
            "size": 0
        });

        let agg_response: ErrorLogsAggregation = self.elastic_obj
            .get_agg_query::<ErrorLogsAggregation>(&search_query, &err_index_name)
            .await
            .context("[ReportServiceImpl->get_agg_err_datas_from_es] The `response body` could not be retrieved.")?;

        let agg_convert_result: Vec<ErrorAggHistoryBucket> = convert_from_histogram_bucket(&agg_response.logs_per_time.buckets)
            .map_err(|e| anyhow!("[MonEsServiceImpl::get_agg_err_datas_from_es] Failed convert agg_response to histogram_bucket {:?}", e))?;

        Ok(agg_convert_result)
    }
}
