use crate::common::*;

use crate::model::{
    message_formatter_dto::message_formatter_urgent::*, monitoring::metric_info::*,
    reports::err_agg_history_bucket::*, search_indicies::*,
};

#[async_trait]
pub trait MonEsService {
    async fn put_node_conn_err_infos(
        &self,
        cluster_name: &str,
        fail_hosts: &[String],
    ) -> anyhow::Result<()>;
    async fn put_cluster_health_unstable_infos(
        &self,
        cluster_name: &str,
        danger_indicies: &[SearchIndicies],
    ) -> anyhow::Result<()>;
    async fn put_urgent_infos(
        &self,
        cluster_name: &str,
        urgent_infos: &[UrgentAlarmInfo],
    ) -> anyhow::Result<()>;
    async fn post_cluster_nodes_infos(&self, metric_infos: Vec<MetricInfo>) -> anyhow::Result<()>;
    async fn get_alarm_urgent_infos(
        &self,
        host_ips: Vec<String>,
    ) -> anyhow::Result<Vec<UrgentAlarmInfo>>;
    async fn get_cluster_err_datas_cnt_from_es(
        &self,
        cluster_name: &str,
        err_title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
    ) -> anyhow::Result<u64>;
    async fn get_agg_err_datas_from_es(
        &self,
        cluster_name: &str,
        err_title: &str,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        calendar_interval: &str,
    ) -> anyhow::Result<Vec<ErrorAggHistoryBucket>>;
}
