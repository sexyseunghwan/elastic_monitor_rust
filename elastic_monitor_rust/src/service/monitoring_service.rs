use crate::common::*;

use crate::traits::service::{
    metric_service_trait::*, mon_es_service_trait::*, monitoring_service_trait::*,
    notification_service_trait::*,
};

use crate::model::{
    message_formatter_dto::{
        message_formatter_index::*, message_formatter_node::*, message_formatter_urgent::*,
    },
    monitoring::metric_info::*,
    search_indicies::*,
};

#[derive(Debug, new)]
pub struct MonitoringServiceImpl<M: MetricService, N: NotificationService, ME: MonEsService> {
    metric_service: Arc<M>,
    notification_service: Arc<N>,
    mon_es_service: Arc<ME>,
}

impl<M, N, ME> MonitoringServiceImpl<M, N, ME>
where
    M: MetricService,
    N: NotificationService,
    ME: MonEsService,
{
    #[doc = "Function that checks whether each node in the cluster has connectivity issues 
             and sends an alarm if problems are detected"]
    async fn cluster_nodes_check(&self) -> Result<(), anyhow::Error> {
        let fail_hosts: Vec<String> = self.metric_service.get_cluster_node_check().await?;

        if !fail_hosts.is_empty() {
            let cluster_name: String = self.metric_service.get_cluster_name().await;
            
            /* Add code that logs errors. */
            self.mon_es_service
                .put_node_conn_err_infos(&cluster_name, &fail_hosts)
                .await
                .map_err(|e| anyhow!("[MonitoringServiceImpl::cluster_nodes_check] {:?}", e))?;

            let msg_fmt: MessageFormatterNode = MessageFormatterNode::new(
                cluster_name,
                fail_hosts.clone(),
                String::from("Elasticsearch Connection Failed"),
                String::from("The connection of these hosts has been LOST."),
            );

            self.notification_service.send_alarm_infos(&msg_fmt).await?;

            /* elasticsearch connection pool rebuild. */ 
            self.metric_service
                .refresh_es_connection_pool(fail_hosts)
                .await
                .map_err(|e| anyhow!("[MonitoringServiceImpl::cluster_nodes_check] {:?}", e))?;
        }
        
        Ok(())
    }

    #[doc = "Function that monityors the cluster's status -> GREEN, YELLOW, RED"]
    async fn cluster_health_check(&self) -> Result<(), anyhow::Error> {
        let health_status: String = self.metric_service.get_cluster_health_check().await?;

        /* If problems occur with the Elasticsearch cluster */
        if health_status == "RED" {
            //if health_status == "GREEN" {
            let cluster_name: String = self.metric_service.get_cluster_name().await;
            let danger_indicies: Vec<SearchIndicies> = self
                .metric_service
                .get_cluster_unstable_index_infos(&cluster_name)
                .await?;
            let all_host: Vec<String> = self.metric_service.get_cluster_all_host_infos().await;

            /* Add code that logs errors. */
            self.mon_es_service
                .put_cluster_health_unstable_infos(&cluster_name, &danger_indicies)
                .await
                .map_err(|e| anyhow!("[MonitoringServiceImpl::cluster_health_check] {:?}", e))?;

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
        
        let metric_infos: Vec<MetricInfo> = self
            .metric_service
            .get_cluster_nodes_infos()
            .await
            .map_err(|e| anyhow!("[MonitoringServiceImpl::input_es_metric_infos] {:?}", e))?;

        self.mon_es_service
            .post_cluster_nodes_infos(metric_infos)
            .await
            .map_err(|e| anyhow!("[MonitoringServiceImpl::input_es_metric_infos] {:?}", e))?;

        /* Pending... */
        /* 모니터링 할 인덱스 metric value 를 서버로 Post -> 당분간 안쓰는 기능 -> 특정 인덱스별로 모니터링 진행함 */
        // match self.metric_service.post_cluster_index_infos().await {
        //     Ok(_) => (),
        //     Err(e) => {
        //         error!("[MainHandler->post_cluster_index_infos] {:?}", e);
        //     }
        // }

        Ok(())
    }

    #[doc = "Emergency Alarm service for critical indicators"]
    async fn send_alarm_urgent_infos(&self) -> Result<(), anyhow::Error> {
        let host_ips: Vec<String> = self.metric_service.extract_host_ips().await;

        let urgent_infos: Vec<UrgentAlarmInfo> = self
            .mon_es_service
            .get_alarm_urgent_infos(host_ips)
            .await
            .map_err(|e| {
                error!(
                    "[MonitoringServiceImpl::send_alarm_urgent_infos][urgent_infos] {:?}",
                    e
                );
                e
            })?;

        if !urgent_infos.is_empty() {
            let cluster_name: String = self.metric_service.get_cluster_name().await;

            /* Add code that logs errors. */
            self.mon_es_service
                .put_urgent_infos(&cluster_name, &urgent_infos)
                .await
                .map_err(|e| anyhow!("[MonitoringServiceImpl::send_alarm_urgent_infos] {:?}", e))?;

            let msg: MessageFormatterUrgent =
                MessageFormatterUrgent::new(cluster_name, urgent_infos);

            self.notification_service.send_alarm_infos(&msg).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl<M, N, ME> MonitoringService for MonitoringServiceImpl<M, N, ME>
where
    M: MetricService + Sync + Send,
    N: NotificationService + Sync + Send,
    ME: MonEsService + Sync + Send,
{
    #[doc = "Function that monitors the Elasticsearch cluster status."]
    async fn monitoring_loop(&self) -> anyhow::Result<()> {
        
        loop {
            
            if let Err(e) = self.cluster_nodes_check().await {
                error!(
                    "[MonitoringServiceImpl::monitoring_loop] cluster_nodes_check() error: {:?}",
                    e
                );
            }

            if let Err(e) = self.cluster_health_check().await {
                error!(
                    "[MonitoringServiceImpl::monitoring_loop] cluster_health_check() error: {:?}",
                    e
                );
            }

            /*
                Partial failures are tolerated to ensure
                that metrics from remaining nodes are still collected even when a specific node becomes unreachable.
            */
            if let Err(e) = self.input_es_metric_infos().await {
                error!(
                    "[MonitoringServiceImpl::monitoring_loop] input_es_metric_infos() error: {:?}",
                    e
                );
            }
            
            if let Err(e) = self.send_alarm_urgent_infos().await {
                error!("[MonitoringServiceImpl->monitoring_loop] send_alarm_urgent_infos() error: {:?}", e);
            }

            std_sleep(Duration::from_secs(10));
        }

    }

    async fn get_cluster_name(&self) -> String {
        self.metric_service.get_cluster_name().await
    }
}
