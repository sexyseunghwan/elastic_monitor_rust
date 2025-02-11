use crate::common::*;

use crate::service::metrics_service::*;

use crate::model::{use_case_config::*, Config::*};

pub struct MainHandler<M: MetricService> {
    metirc_service: M,
}

impl<M: MetricService> MainHandler<M> {
    pub fn new(metirc_service: M) -> Self {
        Self { metirc_service }
    }

    #[doc = "작업 세트"]
    pub async fn task_set(&self) -> Result<(), anyhow::Error> {
        let use_case_config: Arc<UseCaseConfig> = get_usecase_config_info();
        let use_case: &str = use_case_config.use_case().as_str();

        /* 1. 클러스터의 각 노드의 연결 문제가 없는지 살핀다.  */
        self.metirc_service.get_cluster_node_check().await?;

        /* 2. 클러스터의 상태를 살핀다. */
        let health_status: String = self.metirc_service.get_cluster_health_check().await?;

        /*
            3. 클러스터의 상태가 Green이 아니라면 인덱스의 상태를 살핀다.
            - 운영환경/개발환경 코드 분리
        */
        if use_case == "dev" && health_status == "GREEN" {
            self.metirc_service
                .get_cluster_unstable_index_infos(&health_status)
                .await?;
        } else if use_case == "prod" && health_status == "RED" {
            self.metirc_service
                .get_cluster_unstable_index_infos(&health_status)
                .await?;
        } else {
            return Err(anyhow!(
                "[Error][task_set()] Please specify a valid environment (dev/prod)."
            ));
        }

        /* 4. Elasticsearch metric value 서버로 Post */
        self.metirc_service.post_cluster_nodes_infos().await?;

        Ok(())
    }
}
