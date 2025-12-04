use crate::common::*;

use crate::utils_modules::time_utils::*;

use crate::traits::{
    repository::es_repository_trait::*,
    service::{
        chart_service_trait::*, metric_service_trait::*, notification_service_trait::*,
        report_service_trait::*,
    },
};

use crate::enums::{report_range::*, report_type::*};

use crate::repository::es_repository::*;

use crate::model::{
    configs::{config::*, report_config::*},
    elastic_dto::elastic_source_parser::*,
    reports::{err_agg_history_bucket::*, err_log_info::*},
};

#[derive(Debug, new)]
pub struct ReportServiceImpl<M: MetricService, N: NotificationService, C: ChartService> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
    chart_service: Arc<C>,
}

impl<M, N, C> ReportServiceImpl<M, N, C>
where
    M: MetricService,
    N: NotificationService,
    C: ChartService,
{
    #[doc = ""]
    async fn report_cluster_issues(&self, report_type: &ReportType) -> anyhow::Result<()> {
        
        let time_range: ReportRange = match report_type {
            ReportType::Day => ReportType::Day.range(),
            ReportType::Week => ReportType::Week.range(),
            ReportType::Month => ReportType::Month.range(),
            ReportType::Year => ReportType::Year.range(),
        };
        
        // "minute" | "hour" | "day" | "week" | "month"
        let calendar_interval: &str = match report_type {
            ReportType::Day => "minute",
            ReportType::Week => "hour",
            ReportType::Month => "day",
            ReportType::Year => "week",
        };

        let start_at: DateTime<Utc> = time_range.from;
        let end_at: DateTime<Utc> = time_range.to;

        // get err data from elasticsearch
        let err_datas: Vec<ErrorLogInfo> =
            self.get_cluster_err_datas_from_es(start_at, end_at).await?;

        let total_alarm_cnt: i32 = Self::calculate_error_term(&err_datas)?;

        let to_wirte_graph_datas: Vec<ErrorAggHistoryBucket> = self
            .get_agg_err_datas_from_es(start_at, end_at, calendar_interval)
            .await?;

        // 이제 to_wirte_graph_datas 이걸 기준으로 그래프를 만들어주자.
        let img_path: PathBuf = self
            .generate_err_history_graph(report_type, &to_wirte_graph_datas, start_at, end_at)
            .await
            .context("[ReportServiceImpl->report_cluster_issues]")?;

        Ok(())
    }

    #[doc = "Function that returns error log information related to node failures in Elasticsearch"]
    async fn get_cluster_err_datas_from_es(
        &self,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> anyhow::Result<Vec<ErrorLogInfo>> {
        let cluster_name: String = self.metric_service.get_cluster_name();
        let mon_es: ElasticConnGuard = get_elastic_guard_conn().await?;

        let err_index_name: String = format!(
            "{}*",
            mon_es.get_cluster_index_error_pattern()
                .ok_or_else(|| anyhow!("[ReportServiceImpl->get_cluster_err_datas]`Error log index pattern` is not configured"))?
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
                                "err_title.keyword": "Node connection failure"
                            }
                        },
                        {
                            "term": {
                                "cluster_name.keyword": cluster_name
                            }
                        }
                    ]
                }
            },
            "sort": {
                "timestamp": "asc"
            },
            "size": 10000
        });

        let respnse_body: Vec<ErrorLogInfo> = mon_es
            .get_search_query::<ErrorLogInfo>(&search_query, &err_index_name)
            .await
            .context("[ReportServiceImpl->get_cluster_err_datas] The `response body` could not be retrieved.")?;

        Ok(respnse_body)
    }

    #[doc = "Calculate the number of error periods where consecutive errors are more than 60 seconds apart"]
    /// # Arguments
    /// * `err_log_infos` - Slice of error log information sorted by timestamp
    ///
    /// # Returns
    /// * `Ok(i32)` - Number of error periods (gaps > 60 seconds between consecutive errors)
    /// * `Err` - If timestamp parsing fails
    fn calculate_error_term(err_log_infos: &[ErrorLogInfo]) -> anyhow::Result<i32> {
        if err_log_infos.is_empty() {
            return Ok(0);
        }

        let mut err_alarm_cnt: i32 = 0;
        let mut prev_time: Option<DateTime<Utc>> = None;

        for err_log in err_log_infos {
            let err_time: DateTime<Utc> = convert_str_to_datetime(err_log.timestamp(), Utc)?;

            if let Some(prev) = prev_time {
                let time_diff: chrono::TimeDelta = err_time - prev;

                // If gap between errors is more than 60 seconds, it's a new error period
                if time_diff.num_seconds() > 60 {
                    err_alarm_cnt += 1;
                }
            }

            prev_time = Some(err_time);
        }

        Ok(err_alarm_cnt)
    }

    #[doc = "Retrieve aggregated error log data from Elasticsearch using date histogram aggregation"]
    /// # Arguments
    /// * `start_at` - Start time of the query range (UTC)
    /// * `end_at` - End time of the query range (UTC)
    /// * `calendar_interval` - Aggregation interval ("minute", "hour", "day", "week", "month")
    ///
    /// # Returns
    /// * `Ok(Vec<ErrorAggHistoryBucket>)` - List of aggregated error buckets with timestamps converted to Local time
    /// * `Err` - If query fails or data conversion fails
    async fn get_agg_err_datas_from_es(
        &self,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        calendar_interval: &str,
    ) -> anyhow::Result<Vec<ErrorAggHistoryBucket>> {
        let cluster_name: String = self.metric_service.get_cluster_name();
        let mon_es: ElasticConnGuard = get_elastic_guard_conn().await?;

        let err_index_name: String = format!(
            "{}*",
            mon_es.get_cluster_index_error_pattern()
                .ok_or_else(|| anyhow!("[ReportServiceImpl->get_agg_err_datas_from_es]`Error log index pattern` is not configured"))?
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
                                "err_title.keyword": "Node connection failure"
                            }
                        },
                        {
                            "term": {
                                "cluster_name.keyword": cluster_name
                            }
                        }
                    ]
                }
            },
            "aggs": {
                "logs_per_time": {
                    "date_histogram": {
                        "field": "timestamp",
                        "calendar_interval": calendar_interval
                    }
                }
            },
            "size": 0
        });

        let agg_response: ErrorLogsAggregation = mon_es
            .get_agg_query::<ErrorLogsAggregation>(&search_query, &err_index_name)
            .await
            .context("[ReportServiceImpl->get_agg_err_datas_from_es] The `response body` could not be retrieved.")?;

        let agg_convert_result: Vec<ErrorAggHistoryBucket> =
            convert_from_histogram_bucket(&cluster_name, &agg_response.logs_per_time.buckets)?;
        
        Ok(agg_convert_result)
    }

    #[doc = "Generate a line chart visualization of error log history over time"]
    /// # Arguments
    /// * `report_type` - Type of report (Day, Week, Month, Year) - determines output path
    /// * `err_agg_hist_list` - Aggregated error history data with timestamps and counts
    /// * `start_at` - Start time of the data range (UTC)
    /// * `end_at` - End time of the data range (UTC)
    ///
    /// # Returns
    /// * `Ok(PathBuf)` - Path to the generated chart image file
    /// * `Err` - If chart generation fails
    ///
    /// # Notes
    /// - Generates a unique filename using current timestamp and random 6-digit number
    /// - Chart title shows the time range in local timezone
    /// - X-axis shows timestamps, Y-axis shows error counts
    async fn generate_err_history_graph(
        &self,
        report_type: &ReportType,
        err_agg_hist_list: &[ErrorAggHistoryBucket],
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> anyhow::Result<PathBuf> {
        let cur_local_time: DateTime<Local> = Local::now();
        let cur_local_time_str: String = convert_date_to_str_ymdhms(cur_local_time, Local);

        /* Generate 6-digit random number (100000 ~ 999999) */
        let random_6_digit: u32 = {
            let mut rng: ThreadRng = rand::rng();
            rng.random_range(100_000..1_000_000)
        };

        let report_img_path_str: String = match report_type {
            ReportType::Day => get_daily_report_config_info().img_path().to_string(),
            ReportType::Week => get_weekly_report_config_info().img_path().to_string(),
            ReportType::Month => get_monthly_report_config_info().img_path().to_string(),
            ReportType::Year => get_yearly_report_config_info().img_path().to_string(),
        };

        let output_path: PathBuf = PathBuf::from(format!(
            "{}/img_{}{}",
            &report_img_path_str, cur_local_time_str, random_6_digit
        ));

        // 추후에 라벨 기능 수정 해야한다.... x 라벨이 너무 보이니까~~~~
        let x_axis: Vec<String> = err_agg_hist_list
            .iter()
            .map(|eb| convert_date_to_str_full(eb.date_at, Local))
            .collect();

        let y_axis: Vec<i64> = err_agg_hist_list
            .iter()
            .map(|eb| eb.doc_count().clone())
            .collect();

        let agg_start_local_at: String =
            convert_date_to_str_ymd(start_at.with_timezone(&Local), Local);
        let agg_end_local_at: String = convert_date_to_str_ymd(end_at.with_timezone(&Local), Local);

        self.chart_service
            .generate_line_chart(
                &format!(
                    "[{}~{}] Elasticsearch Error Datas",
                    &agg_start_local_at, &agg_end_local_at
                ),
                x_axis,
                y_axis,
                &output_path,
                "timestamp",
                "Error count",
            )
            .await?;

        Ok(output_path)
    }
}

#[async_trait]
impl<M, N, C> ReportService for ReportServiceImpl<M, N, C>
where
    M: MetricService + Sync + Send,
    N: NotificationService + Sync + Send,
    C: ChartService + Sync + Send,
{
    #[doc = "Function that provides a report service"]
    async fn report_loop(&self, report_type: ReportType) -> anyhow::Result<()> {
        //async fn report_loop(&self, report_config: ReportConfig) -> anyhow::Result<()> {

        let report_config: ReportConfig = match report_type {
            ReportType::Day => get_daily_report_config_info().clone(),
            ReportType::Week => get_weekly_report_config_info().clone(),
            ReportType::Month => get_monthly_report_config_info().clone(),
            ReportType::Year => get_yearly_report_config_info().clone(),
        };
        
        let schedule: cron::Schedule = cron::Schedule::from_str(&report_config.cron_schedule)
            .map_err(|e| {
                anyhow!(
                    "[ReportServiceImpl->report_loop] Failed to parse cron schedule '{}': {:?}",
                    report_config.cron_schedule,
                    e
                )
            })?;

        info!(
            "Starting daily report scheduler with cron schedule: {}",
            report_config.cron_schedule
        );

        loop {
            /* The reporting schedule is based on Korean time - GMT+9 */
            let now_local: DateTime<Local> = chrono::Local::now();

            let next_run: DateTime<Local> = schedule
                .upcoming(now_local.timezone())
                .next()
                .ok_or_else(|| anyhow!("[ReportServiceImpl->report_loop] Failed to calculate next run time from cron schedule"))?;

            let duration_until_next_run: Duration = match (next_run - now_local).to_std() {
                Ok(next_run) => next_run,
                Err(e) => {
                    error!(
                        "[MainController->daily_report_loop] Failed to calculate duration: {:?}",
                        e
                    );
                    continue;
                }
            };

            info!(
                "Next report scheduled at: {}. Sleeping for {:?}",
                next_run.format("%Y-%m-%dT%H:%M:%S"),
                duration_until_next_run
            );

            let wake: Instant = Instant::now() + duration_until_next_run;
            sleep_until(wake).await;

            /* Get the current time after waking up */
            let report_time: DateTime<Local> = chrono::Local::now();

            /* The function runs when it's time to send the report email. */
            self.report_cluster_issues(&report_type).await?;
            //self.report_cluster_issues(&report_type).await
            // self.report_index_cnt_task(
            //     mon_index_name,
            //     alarm_index_name,
            //     target_index_info_list,
            //     report_time,
            //     report_type,
            // )
            // .await
            // .unwrap_or_else(|e| {
            //     error!("{:?}", e);
            // });
        }
    }
}
