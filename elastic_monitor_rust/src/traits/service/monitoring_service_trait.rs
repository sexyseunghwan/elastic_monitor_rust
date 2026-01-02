use crate::common::*;

#[async_trait]
pub trait MonitoringService {
    async fn monitoring_loop(&self) -> anyhow::Result<()>;
    async fn get_cluster_name(&self) -> String;
}
