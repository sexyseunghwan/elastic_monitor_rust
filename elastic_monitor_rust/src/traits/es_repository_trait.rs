use crate::common::*;

#[async_trait]
pub trait EsRepository {
    async fn get_indices_info(&self) -> Result<String, anyhow::Error>;
    async fn get_health_info(&self) -> Result<Value, anyhow::Error>;
    async fn get_node_conn_check(&self) -> Vec<(String, bool)>;
    async fn get_node_stats(&self, fields: &[&str]) -> Result<Value, anyhow::Error>;
    async fn get_specific_index_info(&self, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn get_cat_shards(&self, fields: &[&str]) -> Result<String, anyhow::Error>;
    async fn get_cat_thread_pool(&self) -> Result<String, anyhow::Error>;
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error>;
    async fn get_search_query<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Vec<T>, anyhow::Error>;
    fn get_cluster_name(&self) -> String;
    fn get_cluster_all_host_infos(&self) -> Vec<String>;
    fn get_cluster_index_pattern(&self) -> Option<String>;
    fn get_cluster_index_monitoring_pattern(&self) -> Option<String>;
    fn get_cluster_index_urgent_pattern(&self) -> Option<String>;
    fn get_cluster_index_error_pattern(&self) -> Option<String>;
}