use crate::common::*;

use crate::traits::{
    metric_service_trait::*,
    notification_service_trait::*
};

use crate::model::message_formatter_dto::message_formatter_index::*;
use crate::model::message_formatter_dto::message_formatter_node::*;
use crate::model::message_formatter_dto::message_formatter_urgent::*;


use crate::model::{
    indicies::*
};

#[derive(Clone, Debug, new)]
pub struct MainHandler<M: MetricService, N: NotificationService> {
    metirc_service: M,
    notification_service: N
}

impl<M: MetricService, N: NotificationService> MainHandler<M, N> {

    #[doc = "Main 작업 세트"]
    pub async fn main_task_set(&self) -> Result<(), anyhow::Error> {

        /* 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.  */
        self.cluster_nodes_check().await?;
        
        /* 2. 클러스터의 상태를 살핀다. */
        self.cluster_health_check().await?;

        /* 3. Elasitcsearch 모니터링 지표들을 Elaistcsearch 색인 */
        self.input_es_metric_infos().await?;        

        /* 4. 긴급 지표들에 대한 긴급 알람 서비스 */
        self.send_alarm_urgent_infos().await?;
        
        Ok(())
    }
    

    #[doc = "클러스터의 각 노드의 연결 문제가 없는지 살피고 문제가 있다면, 알람을 보내준다."]
    async fn cluster_nodes_check(&self) -> Result<(), anyhow::Error> {
        
        let fail_hosts: Vec<String> = self.metirc_service.get_cluster_node_check().await?;
        
        if !fail_hosts.is_empty() {
            let cluster_name: String = self.metirc_service.get_cluster_name();  
            
            let msg_fmt: MessageFormatterNode = MessageFormatterNode::new(
                cluster_name,
                fail_hosts,
                String::from("Elasticsearch Connection Failed"),
                String::from("The connection of these hosts has been LOST.")
            );

            self.notification_service.send_alarm_infos(&msg_fmt).await?;
        }

        Ok(())
    }

    #[doc = "클러스터의 상태를 모니터링 해주는 함수 -> GREEN, YELLOW, RED 인지"]
    async fn cluster_health_check(&self) -> Result<(), anyhow::Error> {
        let health_status: String = self.metirc_service.get_cluster_health_check().await?;
        
        /* cluster 상태가 문제가 생기는 경우 */
        if health_status == "RED" {
            let danger_indicies: Vec<Indicies> = self.metirc_service.get_cluster_unstable_index_infos().await?;
            let cluster_name: String = self.metirc_service.get_cluster_name();  
            let all_host: Vec<String> = self.metirc_service.get_cluster_all_host_infos();

            let msg_fmt: MessageFormatterIndex = MessageFormatterIndex::new(
                cluster_name,
                all_host, 
                format!(
                    "Elasticsearch Cluster health is [{}]",
                    health_status
                ),
                danger_indicies,
            );

            self.notification_service.send_alarm_infos(&msg_fmt).await?;
        }

        Ok(())
    }
    
    #[doc = "Elasitcsearch 모니터링 지표들을 Elaistcsearch 색인"]
    async fn input_es_metric_infos(&self) -> Result<(), anyhow::Error> {

        /* Elasticsearch metric value 서버로 Post */
        match self.metirc_service.post_cluster_nodes_infos().await {
            Ok(_) => (),
            Err(e) => {
                error!("[ERROR][MainHandler->mainpost_cluster_nodes_infos] {:?}", e);
            }
        }

        /* 모니터링 할 인덱스 metric value 를 서버로 Post */
        match self.metirc_service.post_cluster_index_infos().await {
            Ok(_) => (),
            Err(e) => {
                error!("[ERROR][MainHandler->post_cluster_index_infos] {:?}", e);
            }
        }

        Ok(())
    }   

    #[doc = "긴급 지표들에 대한 긴급 알람 서비스"]
    async fn send_alarm_urgent_infos(&self) -> Result<(), anyhow::Error> {

        let urgent_infos: Vec<UrgentAlarmInfo> = self.metirc_service.get_alarm_urgent_infos().await?;

        if !urgent_infos.is_empty() {
            let cluster_name: String = self.metirc_service.get_cluster_name(); 
            let msg: MessageFormatterUrgent = MessageFormatterUrgent::new(cluster_name, urgent_infos);
            self.notification_service.send_alarm_infos(&msg).await?;
        }

        Ok(())
    }

}