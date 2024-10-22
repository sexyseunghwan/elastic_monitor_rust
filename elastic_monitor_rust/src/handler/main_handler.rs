use crate::common::*;

use crate::service::metric_service::*;


pub struct MainHandler<M: MetricService> {
    metirc_service: M
}


impl<M: MetricService> MainHandler<M> {

    pub fn new(metirc_service: M) -> Self {
        Self {
            metirc_service,
        }
    }
    
    #[doc="Task 세트"]
    pub async fn task_set(&self) -> Result<(), anyhow::Error> {
        
        // 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.
        match self.metirc_service.get_cluster_node_check().await {  
            Ok(flag) => {
                // 노드 연결에 문제가 있는 경우 -> 프로그램 중단.
                if !flag { 
                    return Ok(()) 
                }
            },
            Err(e) => {
                error!("{:?}", e)
            }
        }
        
        // 2. 클러스터의 상태를 살핀다.
        let health_status = self.metirc_service.get_cluster_health_check().await?;
        
        if health_status == "RED" {
            
            // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
            self.metirc_service.get_cluster_unstable_index_infos(&health_status).await?;
            
        } 

        // ==== Pending Task 제외 ====
        // let health_status = self.metirc_service.get_cluster_health_check().await?;
        
        // if health_status == "GREEN" || health_status == "YELLOW" {
            
        //     // 3. pending tasks 가 없는지 확인해준다.
        //     let _pending_task_res = self.metirc_service.get_cluster_pending_tasks().await?;
            
        // } else {
            
        //     // 3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
        //     self.metirc_service.get_cluster_unstable_index_infos(&health_status).await?;
        // }
        
        
        // 4. Elasticsearch metric value 서버로 Post
        self.metirc_service.post_cluster_nodes_infos().await?;

        Ok(())
    }
}