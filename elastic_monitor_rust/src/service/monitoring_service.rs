use crate::common::*;

use crate::traits::service::{
    metric_service_trait::*, monitoring_service_trait::*, notification_service_trait::*,
};

use crate::model::{
    configs::{config::*, report_config::*},
    message_formatter_dto::{
        message_formatter_index::*, message_formatter_node::*, message_formatter_urgent::*,
    },
    search_indicies::*,
};

#[derive(Debug, new)]
pub struct MonitoringServiceImpl<M: MetricService, N: NotificationService> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
}

impl<M, N> MonitoringServiceImpl<M, N>
where
    M: MetricService,
    N: NotificationService,
{
    #[doc = "Function that checks whether each node in the cluster has connectivity issues 
             and sends an alarm if problems are detected"]
    async fn cluster_nodes_check(&self) -> Result<(), anyhow::Error> {
        let fail_hosts: Vec<String> = self.metric_service.get_cluster_node_check().await?;
        
        if !fail_hosts.is_empty() {
            let cluster_name: String = self.metric_service.get_cluster_name();

            /* Add code that logs errors. */
            self.metric_service
                .put_node_conn_err_infos(&cluster_name, &fail_hosts)
                .await?;

            let msg_fmt: MessageFormatterNode = MessageFormatterNode::new(
                cluster_name,
                fail_hosts,
                String::from("Elasticsearch Connection Failed"),
                String::from("The connection of these hosts has been LOST."),
            );
            
            self.notification_service.send_alarm_infos(&msg_fmt).await?;
        }

        Ok(())
    }

    #[doc = "Function that monityors the cluster's status -> GREEN, YELLOW, RED"]
    async fn cluster_health_check(&self) -> Result<(), anyhow::Error> {
        let health_status: String = self.metric_service.get_cluster_health_check().await?;

        /* If problems occur with the Elasticsearch cluster */
        if health_status == "RED" {
            let cluster_name: String = self.metric_service.get_cluster_name();
            let danger_indicies: Vec<SearchIndicies> = self
                .metric_service
                .get_cluster_unstable_index_infos(&cluster_name)
                .await?;
            let all_host: Vec<String> = self.metric_service.get_cluster_all_host_infos();

            /* Add code that logs errors. */
            self.metric_service
                .put_cluster_health_unstable_infos(&cluster_name, &danger_indicies)
                .await?;
            
            let msg_fmt: MessageFormatterIndex = MessageFormatterIndex::new(
                cluster_name,
                all_host,
                format!("Elasticsearch Cluster health is [{}]", health_status),
                danger_indicies,
            );

            self.notification_service.send_alarm_infos(&msg_fmt).await?;
        }

        Ok(())
    }

    #[doc = "Function that indexes observation metrics into a specific index 
             within an Elasticsearch cluster responsible for monitoring"]
    async fn input_es_metric_infos(&self) -> Result<(), anyhow::Error> {
        /* Elasticsearch metric value 서버로 Post */
        match self.metric_service.post_cluster_nodes_infos().await {
            Ok(_) => (),
            Err(e) => {
                error!("[MainHandler->mainpost_cluster_nodes_infos] {:?}", e);
            }
        }

        /* 모니터링 할 인덱스 metric value 를 서버로 Post */
        match self.metric_service.post_cluster_index_infos().await {
            Ok(_) => (),
            Err(e) => {
                error!("[MainHandler->post_cluster_index_infos] {:?}", e);
            }
        }

        Ok(())
    }

    #[doc = "Emergency Alarm service for critical indicators"]
    async fn send_alarm_urgent_infos(&self) -> Result<(), anyhow::Error> {
        let urgent_infos: Vec<UrgentAlarmInfo> = self
            .metric_service
            .get_alarm_urgent_infos()
            .await
            .map_err(|e| {
                error!(
                    "[MainHandler->send_alarm_urgent_infos->get_alarm_urgent_infos] {:?}",
                    e
                );
                e
            })?;

        if !urgent_infos.is_empty() {
            let cluster_name: String = self.metric_service.get_cluster_name();

            /* Add code that logs errors. */
            self.metric_service
                .put_urgent_infos(&cluster_name, &urgent_infos)
                .await?;

            let msg: MessageFormatterUrgent =
                MessageFormatterUrgent::new(cluster_name, urgent_infos);

            self.notification_service.send_alarm_infos(&msg).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl<M, N> MonitoringService for MonitoringServiceImpl<M, N>
where
    M: MetricService + Sync + Send,
    N: NotificationService + Sync + Send,
{
    #[doc = ""]
    async fn monitoring_loop(&self) -> anyhow::Result<()> {
        loop {
            match self.cluster_nodes_check().await {
                Ok(_) => (),
                Err(e) => {
                    error!("[MonitoringServiceImpl->monitoring_loop] cluster_nodes_check() error: {:?}", e);
                    continue;
                }
            };

            match self.cluster_health_check().await {
                Ok(_) => (),
                Err(e) => {
                    error!("[MonitoringServiceImpl->monitoring_loop] cluster_health_check() error: {:?}", e);
                    continue;
                }
            }

            match self.input_es_metric_infos().await {
                Ok(_) => (),
                Err(e) => {
                    error!("[MonitoringServiceImpl->monitoring_loop] input_es_metric_infos() error: {:?}", e);
                    continue;
                }
            }

            match self.send_alarm_urgent_infos().await {
                Ok(_) => (),
                Err(e) => {
                    error!("[MonitoringServiceImpl->monitoring_loop] send_alarm_urgent_infos() error: {:?}", e);
                    continue;
                }
            }

            std_sleep(Duration::from_secs(10));
        }
    }

    fn get_cluster_name(&self) -> String {
        self.metric_service.get_cluster_name()
    }
}
