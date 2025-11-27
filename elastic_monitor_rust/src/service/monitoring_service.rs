use crate::common::*;

use crate::traits::service::{metric_service_trait::*, notification_service_trait::*};

#[derive(Debug, new)]
pub struct MonitorinServiceImpl<M: MetricService, N: NotificationService> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
}

impl<M, N> MonitorinServiceImpl<M, N>
where
    M: MetricService,       //+ Sync + Send,
    N: NotificationService, //+ Sync + Send,
{
    
}
