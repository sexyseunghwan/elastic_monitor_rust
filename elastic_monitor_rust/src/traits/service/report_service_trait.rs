use crate::common::*;

use crate::enums::report_type::*;

use crate::model::configs::{
    config::*,
    report_config::*
};



#[async_trait]
pub trait ReportService {
    async fn report_loop(
        &self,
        report_config: ReportConfig
    ) -> anyhow::Result<()>;
}