use crate::common::*;

use crate::traits::service::{
    metric_service_trait::*, notification_service_trait::*, report_service_trait::*,
};

use crate::enums::report_type::*;

use crate::model::configs::{config::*, report_config::*};

#[derive(Debug, new)]
pub struct ReportServiceImpl<M: MetricService, N: NotificationService> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
}

impl<M, N> ReportServiceImpl<M, N>
where
    M: MetricService,
    N: NotificationService,
{
    #[doc = ""]
    async fn report_cluster_issues(&self) -> anyhow::Result<()> {
        
        
        
        Ok(())
    }
}

#[async_trait]
impl<M, N> ReportService for ReportServiceImpl<M, N>
where
    M: MetricService + Sync + Send,
    N: NotificationService + Sync + Send,
{
    #[doc = "Function that provides a report service"]
    async fn report_loop(&self, report_config: ReportConfig) -> anyhow::Result<()> {
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
