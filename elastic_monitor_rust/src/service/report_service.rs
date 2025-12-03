use toml::value::Datetime;

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
    async fn report_cluster_issues(
        &self,
        report_range: ReportRange,
        report_type: ReportType,
    ) -> anyhow::Result<()> {
        // "minute" | "hour" | "day" | "week" | "month"
        let calendar_interval: &str = match report_type {
            ReportType::Day => "minute",
            ReportType::Week => "hour",
            ReportType::Month => "day",
            ReportType::Year => "week",
        };

        let start_at: DateTime<Utc> = report_range.from;
        let end_at: DateTime<Utc> = report_range.to;

        // get err data from elasticsearch
        let err_datas: Vec<ErrorLogInfo> =
            self.get_cluster_err_datas_from_es(start_at, end_at).await?;

        let total_alarm_cnt: i32 = Self::calculate_error_term(&err_datas)?;
        let to_wirte_graph_datas: Vec<ErrorAggHistoryBucket> = self
            .get_agg_err_datas_from_es(start_at, end_at, calendar_interval)
            .await?;

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

    #[doc = ""]
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
                },
                "aggs": {
                    "logs_per_time": {
                        "date_histogram": {
                            "field": "timestamp",
                            "calendar_interval": calendar_interval
                        }
                    }
                }
            },
            "size": 0
        });

        let respnse_body: Vec<DateHistogramBucket> = mon_es
            .get_search_query::<DateHistogramBucket>(&search_query, &err_index_name)
            .await
            .context("[ReportServiceImpl->get_agg_err_datas_from_es] The `response body` could not be retrieved.")?;

        let agg_convert_result: Vec<ErrorAggHistoryBucket> =
            convert_from_histogram_bucket(&cluster_name, &respnse_body)?;

        Ok(agg_convert_result)
    }
    
    
    #[doc = ""]
    async fn generate_err_history_graph(
        &self,
        err_agg_hist_list: &[ErrorAggHistoryBucket],
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        
    ) -> anyhow::Result<()> {

        


        Ok(())
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

        let time_range: ReportRange = match report_type {
            ReportType::Day => ReportType::Day.range(),
            ReportType::Week => ReportType::Week.range(),
            ReportType::Month => ReportType::Month.range(),
            ReportType::Year => ReportType::Year.range(),
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
