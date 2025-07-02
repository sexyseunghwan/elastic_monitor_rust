use crate::common::*;

use crate::model::index_metric_info::*;
use crate::model::message_formatter_dto::message_formatter::*;
use crate::model::metric_info::*;

#[async_trait]
pub trait MetricService {
    async fn get_cluster_node_check(&self) -> Result<(), anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(
        &self,
        cluster_status: &str,
    ) -> Result<(), anyhow::Error>;
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
    async fn send_alarm_infos<T: MessageFormatter + Sync + Send>(
        &self,
        msg_fmt: &T,
    ) -> Result<(), anyhow::Error>;
    async fn post_cluster_index_infos(&self) -> Result<(), anyhow::Error>;
    async fn send_alarm_urgent_infos(&self) -> Result<(), anyhow::Error>;
}