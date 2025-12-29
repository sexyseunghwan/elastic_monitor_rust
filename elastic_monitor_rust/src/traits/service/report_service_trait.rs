use crate::common::*;

use crate::enums::report_type::*;

#[async_trait]
pub trait ReportService {
    async fn report_loop(&self, report_type: ReportType, cluster_name: &str) -> anyhow::Result<()>;
}
