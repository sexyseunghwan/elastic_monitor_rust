use crate::common::*;

#[async_trait]
pub trait MonitoringService {
    async fn monitoring_loop(&self) -> anyhow::Result<()>;
}
