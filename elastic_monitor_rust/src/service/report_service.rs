use futures::TryFutureExt;

use crate::common::*;

use crate::utils_modules::{io_utils::*, time_utils::*};

use crate::traits::service::{
        chart_service_trait::*, metric_service_trait::*, mon_es_service_trait::*,
        notification_service_trait::*, report_service_trait::*,
    };

use crate::enums::{report_type::*, img_file_type::*};

use crate::env_configuration::env_config::*;

use crate::model::{
    configs::{config::*, report_config::*},
    reports::err_agg_history_bucket::*,
    reports::report_range::*
};

#[derive(Debug, new)]
pub struct ReportServiceImpl<
    M: MetricService,
    N: NotificationService,
    C: ChartService,
    ME: MonEsService,
> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
    chart_service: Arc<C>,
    mon_es_service: Arc<ME>,
}

impl<M, N, C, ME> ReportServiceImpl<M, N, C, ME>
where
    M: MetricService,
    N: NotificationService,
    C: ChartService,
    ME: MonEsService,
{
    #[doc = ""]
    async fn report_cluster_issues(
        &self,
        report_type: &ReportType,
        cluster_name: &str,
    ) -> anyhow::Result<()> {
        let time_range: ReportRange = report_type.range();

        let calendar_interval: &str = match report_type {
            ReportType::Day => "minute",
            ReportType::Week => "hour",
            ReportType::Month => "day",
            ReportType::Year => "week",
        };

        let start_at: DateTime<Utc> = time_range.from;
        let end_at: DateTime<Utc> = time_range.to;
        
        /* Node connection failure */
        let (con_err_cnt, con_err_agg_img_path) = self
            .process_error_type(
                ImgFileType::NodeConnErr,
                "Node connection failure",
                cluster_name,
                report_type,
                start_at,
                end_at,
                calendar_interval,
            )
            .await
            .map_err(|e| {
                anyhow!(
                    "[ReportServiceImpl::report_cluster_issues] node connection fail: {:?}",
                    e
                )
            })?;

        /* Cluster status is unstable */
        let (unstable_cnt, unstable_agg_img_path) = self
            .process_error_type(
                ImgFileType::ClusterStatusErr,
                "Cluster status is unstable",
                cluster_name,
                report_type,
                start_at,
                end_at,
                calendar_interval,
            )
            .await
            .map_err(|e| {
                anyhow!(
                    "[ReportServiceImpl::report_cluster_issues] cluster status unstable: {:?}",
                    e
                )
            })?;

        /* Emergency indicator alarm dispatch */
        let (emergency_cnt, emergency_agg_img_path) = self
            .process_error_type(
                ImgFileType::EmgIndiErr,
                "Emergency indicator alarm dispatch",
                cluster_name,
                report_type,
                start_at,
                end_at,
                calendar_interval,
            )
            .await
            .map_err(|e| {
                anyhow!(
                    "[ReportServiceImpl::report_cluster_issues] emergency indicators: {:?}",
                    e
                )
            })?;

        let local_start_at: DateTime<Local> = start_at.with_timezone(&Local);
        let local_end_at: DateTime<Local> = end_at.with_timezone(&Local);

        let html_content: String = self
            .generate_report_html(
                report_type,
                local_start_at,
                local_end_at,
                con_err_cnt,
                unstable_cnt,
                emergency_cnt,
                &con_err_agg_img_path,
                &unstable_agg_img_path,
                &emergency_agg_img_path,
            )
            .await?;

        /* Send the report via email. */
        let email_subject: String = format!(
            "[Elasticsearch] {} error Report - {}",
            report_type.get_name(),
            cluster_name
        );

        self.notification_service
            .send_alert_infos_to_admin(&email_subject, &html_content)
            .await?;
        
        delete_files_if_exists(vec![
            con_err_agg_img_path,
            unstable_agg_img_path,
            emergency_agg_img_path,
        ])?;

        Ok(())
    }

    #[doc = "Process error data for a specific error type: count, aggregate, and generate graph"]
    /// # Returns
    /// * `Ok((u64, PathBuf))` - Error count and generated image path
    async fn process_error_type(
        &self,
        img_file_type: ImgFileType,
        err_title: &str,
        cluster_name: &str,
        report_type: &ReportType,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        calendar_interval: &str,
    ) -> anyhow::Result<(u64, PathBuf)> {
        
        let err_cnt: u64 = self
            .mon_es_service
            .get_cluster_err_datas_cnt_from_es(cluster_name, err_title, start_at, end_at)
            .await
            .map_err(|e| anyhow!("[ReportServiceImpl::process_error_type][err_cnt] {:?}", e))?;

        let agg_list: Vec<ErrorAggHistoryBucket> = self
            .mon_es_service
            .get_agg_err_datas_from_es(cluster_name, err_title, start_at, end_at, calendar_interval)
            .await?;

        let img_path: PathBuf = self
            .generate_err_history_graph(
                img_file_type,
                cluster_name,
                report_type,
                &agg_list,
                start_at,
                end_at,
                err_title,
            )
            .await
            .with_context(|| {
                format!(
                    "[ReportServiceImpl->process_error_type] Failed to generate graph for '{}'",
                    err_title
                )
            })?;

        Ok((err_cnt, img_path))
    }

    #[doc = "Generate a line chart visualization of error log history over time"]
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
        img_file_type: ImgFileType,
        cluster_name: &str,
        report_type: &ReportType,
        err_agg_hist_list: &[ErrorAggHistoryBucket],
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        img_subject: &str,
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
            "{}img_{}_{}_{}_{}.png",
            &report_img_path_str, cluster_name, cur_local_time_str, img_file_type.get_name(), random_6_digit
        ));

        let x_axis: Vec<String> = err_agg_hist_list
            .iter()
            .map(|eb| convert_date_to_str_full(eb.date_at, Local))
            .collect();

        let y_axis: Vec<i64> = err_agg_hist_list
            .iter()
            .map(|eb| eb.doc_count().clone())
            .collect();

        let agg_start_local_at: String =
            convert_date_to_str_ymd_mail(start_at.with_timezone(&Local), Local);
        let agg_end_local_at: String =
            convert_date_to_str_ymd_mail(end_at.with_timezone(&Local), Local);

        self.chart_service
            .generate_line_chart(
                &format!(
                    "[{}~{}] {}",
                    &agg_start_local_at, &agg_end_local_at, img_subject
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

    #[doc = ""]
    async fn generate_report_html(
        &self,
        report_type: &ReportType,
        start_local_datetime: DateTime<Local>,
        end_local_datetime: DateTime<Local>,
        node_conn_fail_cnt: u64,
        cluster_unstable_cnt: u64,
        urgent_indicator_cnt: u64,
        node_conn_fail_chart_img_path: &PathBuf,
        cluster_unstable_chart_img_path: &PathBuf,
        urgent_indicator_chart_img_path: &PathBuf,
    ) -> anyhow::Result<String> {
        let now_local: DateTime<Local> = Local::now();

        let total_alert_cnt: u64 = urgent_indicator_cnt;
        let total_disable_cnt: u64 = node_conn_fail_cnt + cluster_unstable_cnt;
        let total_alarm_cnt: u64 = total_alert_cnt + total_disable_cnt;

        /* Read HTML template file */
        let template_content: String = std::fs::read_to_string(&*REPORT_HTML_TEMPLATE_PATH)
            .map_err(|e| {
                anyhow!(
                    "[MainController->generate_daily_report_html] Failed to read template: {:?}",
                    e
                )
            })?;

        let report_type: String = report_type.get_name(); // Daily, Weekly, Monthly...

        let agg_interval: String = format!(
            "{}~{}",
            convert_date_to_str_human(start_local_datetime, Local),
            convert_date_to_str_human(end_local_datetime, Local)
        );

        let node_conn_fail_chart_img: String = self
            .chart_service
            .convert_images_to_base64_html(&node_conn_fail_chart_img_path)
            .await?;

        let cluster_unstable_chart_img: String = self
            .chart_service
            .convert_images_to_base64_html(&cluster_unstable_chart_img_path)
            .await?;

        let urgent_indicator_chart_img: String = self
            .chart_service
            .convert_images_to_base64_html(&urgent_indicator_chart_img_path)
            .await?;

        let html_content: String = template_content
            .replace("{{REPORT_TYPE}}", &report_type)
            .replace("{{REPORT_INTERVAL}}", &agg_interval)
            .replace(
                "{{REPORT_DATE}}",
                &convert_date_to_str_human(now_local, Local),
            )
            .replace("{{TOTAL_ALERT_CNT}}", &total_alert_cnt.to_string())
            .replace("{{NODE_CONN_FAIL_CNT}}", &node_conn_fail_cnt.to_string())
            .replace("{{TOTAL_DISABLE_CNT}}", &total_disable_cnt.to_string())
            .replace("{{CHANGE_STYLE}}", "")
            .replace(
                "{{CLUSTER_UNSTABLE_CNT}}",
                &cluster_unstable_cnt.to_string(),
            )
            .replace("{{TOTAL_ALARM_CNT}}", &total_alarm_cnt.to_string())
            .replace(
                "{{URGENT_INDICATOR_CNT}}",
                &urgent_indicator_cnt.to_string(),
            )
            .replace(
                "{{NODE_CONN_FAIL_CHART_IMG}}",
                &node_conn_fail_chart_img.to_string(),
            )
            .replace(
                "{{CLUSTER_UNSTABLE_CHART_IMG}}",
                &cluster_unstable_chart_img.to_string(),
            )
            .replace(
                "{{URGENT_INDICATOR_CHART_IMG}}",
                &urgent_indicator_chart_img.to_string(),
            );

        Ok(html_content)
    }
}

#[async_trait]
impl<M, N, C, ME> ReportService for ReportServiceImpl<M, N, C, ME>
where
    M: MetricService + Sync + Send,
    N: NotificationService + Sync + Send,
    C: ChartService + Sync + Send,
    ME: MonEsService + Sync + Send,
{
    #[doc = "Function that provides a report service"]
    async fn report_loop(&self, report_type: ReportType, cluster_name: &str) -> anyhow::Result<()> {
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
            //let report_time: DateTime<Local> = chrono::Local::now(); // 애 따로 필요없을 것 같긴한데...?!...

            /* The function runs when it's time to send the report email. */
            self.report_cluster_issues(&report_type, cluster_name)
                .await?;
        }
    }
}
