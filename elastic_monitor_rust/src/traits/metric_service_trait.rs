use crate::common::*;

use crate::model::{
    index_metric_info::*,
    message_formatter_dto::message_formatter_urgent::*,
    monitoring::metric_info::*,
    indicies::*,
    err_log_dto::{
        err_log_info::*
    }
};


#[async_trait]
pub trait MetricService {
    fn get_cluster_name(&self) -> String;
    fn get_cluster_all_host_infos(&self) -> Vec<String>;
    async fn get_cluster_node_check(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(&self) -> Result<Vec<Indicies>, anyhow::Error>;
    async fn get_nodes_stats_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
        cur_utc_time_str: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_index_stats_handle(
        &self,
        index_name: &str,
        cur_utc_time_str: &str,
    ) -> Result<IndexMetricInfo, anyhow::Error>;
    async fn get_cat_shards_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error>;
    async fn get_cat_thread_pool_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error>;
    async fn post_cluster_nodes_infos(&self) -> Result<(), anyhow::Error>;
    async fn post_cluster_index_infos(&self) -> Result<(), anyhow::Error>;
    async fn get_alarm_urgent_infos(&self) -> Result<Vec<UrgentAlarmInfo>, anyhow::Error>;
    async fn put_node_conn_err_infos(&self, cluster_name: &str, fail_hosts: &[String]) -> anyhow::Result<()>;
    //async fn put_error_logs(&self, error_log_info: ErrorLogInfo) -> anyhow::Result<()>;
}