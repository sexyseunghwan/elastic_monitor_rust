use crate::common::*;

use crate::model::{
    monitoring::metric_info::*,
    search_indicies::*,
};

#[async_trait]
pub trait MetricService {
    async fn get_cluster_name(&self) -> String;
    async fn get_cluster_all_host_infos(&self) -> Vec<String>;
    async fn get_cluster_node_check(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn get_cluster_health_check(&self) -> Result<String, anyhow::Error>;
    async fn get_cluster_unstable_index_infos(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<SearchIndicies>, anyhow::Error>;
    async fn get_nodes_stats_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
        cur_utc_time_str: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_cat_shards_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error>;
    async fn get_cat_thread_pool_handle(
        &self,
        metric_vec: &mut Vec<MetricInfo>,
    ) -> Result<(), anyhow::Error>;
    async fn get_cluster_nodes_infos(&self) -> anyhow::Result<Vec<MetricInfo>>;
    async fn extract_host_ips(&self) -> Vec<String>;
    async fn refresh_es_connection_pool(&self, disable_node_list: Vec<String>) -> anyhow::Result<()>; // -> ?
}
