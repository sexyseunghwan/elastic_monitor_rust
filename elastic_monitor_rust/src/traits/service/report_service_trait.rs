use crate::common::*;

#[async_trait]
pub trait ReportService {
    async fn report_loop(&self) -> anyhow::Result<()>;
}
