use crate::common::*;

use crate::traits::service::{monitoring_service_trait::*, report_service_trait::*};

use crate::model::configs::config::*;

use crate::enums::report_type::*;

use crate::utils_modules::io_utils::*;

#[derive(Debug, new)]
pub struct MainController<M: MonitoringService, R: ReportService> {
    monitoring_service: Arc<M>,
    report_service: Arc<R>,
}

impl<M, R> MainController<M, R>
where
    M: MonitoringService + Send + Sync + 'static,
    R: ReportService + Send + Sync + 'static,
{
    #[doc = "Function that handles both the monitoring system and the reporting system."]
    pub async fn main_task(&self) -> anyhow::Result<()> {
        /* Monitoring tasks and reporting tasks are executed in parallel. */
        /* Cluster name to be monitored */
        let cluster_name: String = self.monitoring_service.get_cluster_name().await;

        /* 1. Monitoring task */
        let monitoring_handle: tokio::task::JoinHandle<()> =
            Self::spawn_monitoring_task(Arc::clone(&self.monitoring_service), &cluster_name);

        /* 2. Report Tasks list */
        let daily_enabled: bool = get_daily_report_config_info().enabled;
        let weekly_enabled: bool = get_weekly_report_config_info().enabled;
        let monthly_enabled: bool = get_monthly_report_config_info().enabled;
        let yearly_enabled: bool = get_yearly_report_config_info().enabled;

        /* 1. Daily report task */
        let daily_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            ReportType::Day,
            "daily_report_task",
            daily_enabled,
            &cluster_name,
        );

        /* 2. Weekly report task */
        let weekly_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            ReportType::Week,
            "weekly_report_task",
            weekly_enabled,
            &cluster_name,
        );

        /* 3. Monthly report task */
        let monthly_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            ReportType::Month,
            "monthly_report_task",
            monthly_enabled,
            &cluster_name,
        );

        /* 4. Yearly report task */
        let yearly_report_handle = Self::spawn_report_task(
            Arc::clone(&self.report_service),
            ReportType::Year,
            "yearly_report_task",
            yearly_enabled,
            &cluster_name,
        );

        /* Run all tasks in parallel and wait for termination */
        let _ = tokio::join!(
            monitoring_handle,
            daily_report_handle,
            weekly_report_handle,
            monthly_report_handle,
            yearly_report_handle
        );

        Ok(())
    }

    #[doc = "Spawn monitoring task as a separate tokio task"]
    fn spawn_monitoring_task(service: Arc<M>, cluster_name: &str) -> tokio::task::JoinHandle<()>
    where
        M: MonitoringService,
    {
        let task_name: String = format!("monitoring_task_{}", cluster_name);

        tokio::spawn(async move {
            match service.monitoring_loop().await {
                Ok(_) => info!(
                    "[spawn_monitoring_task->{}] Completed successfully",
                    task_name
                ),
                Err(e) => error!("[{}] Failed with error: {:?}", task_name, e),
            }
        })
    }

    #[doc = "Spawn report task as a separate tokio task"]
    fn spawn_report_task(
        service: Arc<R>,
        report_type: ReportType,
        task_name: &str,
        enabled: bool,
        cluster_name: &str,
    ) -> tokio::task::JoinHandle<()>
    where
        R: ReportService,
    {
        let task_name: String = task_name.to_string();
        let cluster_name_cloned: String = cluster_name.to_string();

        let img_path: &str = match report_type {
            ReportType::Day => get_daily_report_config_info().img_path().as_str(),
            ReportType::Week => get_weekly_report_config_info().img_path().as_str(),
            ReportType::Month => get_monthly_report_config_info().img_path().as_str(),
            ReportType::Year => get_yearly_report_config_info().img_path().as_str(),
        };
        
        /* It deletes all image files related to the report. */
        match delete_all_files_in_directory(img_path) {
            Ok(_) => {
                info!(
                    "The images within the `{}` directory have been deleted.",
                    img_path
                );
            }
            Err(e) => {
                error!("[MainController::spawn_report_task] Failed to delete contents within the `{}` directory.: {:?}", img_path, e);
            }
        }

        if !enabled {
            return tokio::spawn(async move {
                info!("[{}] Disabled. Skipping.", task_name);
            });
        }

        tokio::spawn(async move {
            match service
                .report_loop(report_type, cluster_name_cloned.as_str())
                .await
            {
                Ok(_) => info!("[spawn_report_task->{}] Completed successfully", task_name),
                Err(e) => error!(
                    "[spawn_report_task->{}] Failed with error [cluster name: {}]: {:?}",
                    task_name, cluster_name_cloned, e
                ),
            }
        })
    }
}