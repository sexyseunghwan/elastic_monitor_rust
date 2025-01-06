use crate::common::*;
use crate::model::ClusterConfig::*;
use crate::utils_modules::io_utils::*;

#[doc = "Elasticsearch DB 초기화"]
/// # Returns
/// * Result<Vec<EsRepositoryPub>, anyhow::Error> - 모니터링 할 대상 Elasticsearch 정보 list
pub fn initialize_db_clients() -> Result<Vec<EsRepositoryPub>, anyhow::Error> {
    let mut elastic_conn_vec: Vec<EsRepositoryPub> = Vec::new();

    let cluster_config: ClusterConfig =
        read_toml_from_file::<ClusterConfig>("./config/elastic_server_info.toml")?;

    for config in &cluster_config.clusters {
        let es_helper = EsRepositoryPub::new(
            &config.cluster_name,
            config.hosts.clone(),
            &config.es_id,
            &config.es_pw,
            &config.index_pattern,
        )?;

        elastic_conn_vec.push(es_helper);
    }

    Ok(elastic_conn_vec)
}

#[async_trait]
pub trait EsRepository {
    async fn get_indices_info(&self) -> Result<String, anyhow::Error>;
    async fn get_health_info(&self) -> Result<Value, anyhow::Error>;
    async fn get_pendging_tasks(&self) -> Result<Value, anyhow::Error>;
    async fn get_node_conn_check(&self) -> Vec<(String, bool)>;
    async fn get_node_stats(&self, fields: &[&str]) -> Result<Value, anyhow::Error>;

    async fn get_cat_shards(&self, fields: &[&str]) -> Result<String, anyhow::Error>;
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error>;

    fn get_cluster_name(&self) -> String;
    fn get_cluster_all_host_infos(&self) -> Vec<String>;
    fn get_cluster_index_pattern(&self) -> String;
}

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsRepositoryPub {
    pub cluster_name: String,
    pub es_clients: Vec<Arc<EsClient>>,
    pub index_pattern: String,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryPub {
    #[doc = "Elasticsearch connection 생성자"]
    /// # Arguments
    /// * `cluster_name`        - Elasticsearch Cluster 이름
    /// * `hosts`               - Elasticsearch host 주소 벡터
    /// * `es_id`               - Elasticsearch 계정정보 - 아이디
    /// * `es_pw`               - Elasticsearch 계정정보 - 비밀번호
    /// * `log_index_pattern`   - Elasticsearch 의 지표정보를 저장해줄 인덱스 패턴 이름
    ///
    /// # Returns
    /// * Result<Self, anyhow::Error>
    pub fn new(
        cluster_name: &str,
        hosts: Vec<String>,
        es_id: &str,
        es_pw: &str,
        log_index_pattern: &str,
    ) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<Arc<EsClient>> = Vec::new();

        for url in hosts {
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);

            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()?;

            let elastic_conn = Elasticsearch::new(transport);
            let es_client = Arc::new(EsClient::new(url, elastic_conn));
            es_clients.push(es_client);
        }

        Ok(EsRepositoryPub {
            cluster_name: cluster_name.to_string(),
            es_clients,
            index_pattern: log_index_pattern.to_string(),
        })
    }

    #[doc = "Common logic: common node failure handling and node selection"]
    /// # Arguments
    /// * `operation` - 실행할 함수 trait
    ///
    /// # Returns
    /// * Result<Response, anyhow::Error>
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(Arc<EsClient>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;

        /* StdRng를 사용하여 Send 트레잇 문제 해결 - 랜덤 시드로 생성 */
        let mut rng = StdRng::from_entropy();

        /* 클라이언트 목록을 셔플 */
        let mut shuffled_clients: Vec<Arc<EsClient>> = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng); /* StdRng를 사용하여 셔플 */

        /* 셔플된 클라이언트들에 대해 순차적으로 operation 수행 */
        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        /* 모든 노드에서 실패했을 경우 에러 반환 */
        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}

#[async_trait]
impl EsRepository for EsRepositoryPub {
    #[doc = "Elasticsearch 클러스터 내부에 존재하는 인덱스들의 정보를 가져오는 함수"]
    /// # Returns
    /// * Result<String, anyhow::Error> - 클러스터 내에 존재하는 각 인덱스들의 이름 및 health 정보
    async fn get_indices_info(&self) -> Result<String, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                let response = es_client
                    .es_conn
                    .cat()
                    .indices(CatIndicesParts::None)
                    .h(&["health", "status", "index"])
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let response_body: String = response.text().await?;
            Ok(response_body)
        } else {
            let error_message = format!(
                "[Elasticsearch Error][get_indices_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 클러스터의 Health Check 해주는 함수."]
    async fn get_health_info(&self) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                /* _cluster/health 요청 */
                let response = es_client
                    .es_conn
                    .cluster()
                    .health(ClusterHealthParts::None)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message = format!(
                "[Elasticsearch Error][get_health_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 의 pending task(중단작업) 가 있는지 확인해주는 함수."]
    async fn get_pendging_tasks(&self) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                /* _cluster/pending_tasks 요청 */
                let response = es_client.es_conn.cluster().pending_tasks().send().await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message = format!("[Elasticsearch Error][get_pendging_tasks()] Failed to GET document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 각 노드들이 현재 문제 없이 통신이 되는지 체크해주는 함수."]
    /// # Returns
    /// * Vec<(String, bool)> -
    async fn get_node_conn_check(&self) -> Vec<(String, bool)> {
        let futures = self.es_clients.iter().map(|es_obj| {
            let es_host = es_obj.host.clone();
            let es_pool = es_obj.es_conn.clone();

            async move {
                let response = es_pool.ping().send().await;
                let is_success = match response {
                    Ok(res) if res.status_code().is_success() => true,
                    _ => false,
                };

                (es_host, is_success)
            }
        });

        join_all(futures).await
    }

    #[doc = "클러스터 각 노드의 metric value 를 반환해주는 함수."]
    /// # Arguments
    /// * `fields` - 모니터링 대상이 되는 지표항목
    ///
    /// # Returns
    /// * Result<Value, anyhow::Error>
    async fn get_node_stats(&self, fields: &[&str]) -> Result<Value, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                /* _nodes/stats 요청 */
                let response = es_client
                    .es_conn
                    .nodes()
                    .stats(NodesStatsParts::None)
                    .fields(fields)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message = format!(
                "[Elasticsearch Error][get_node_stats()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "GET /_cat/shards"]
    /// # Arguments
    /// * `fields` - 모니터링 대상이 되는 지표항목
    ///
    /// # Returns
    /// * Result<Value, anyhow::Error>
    async fn get_cat_shards(&self, fields: &[&str]) -> Result<String, anyhow::Error> {
        let response = self
            .execute_on_any_node(|es_client| async move {
                /* _nodes/stats 요청 */
                let response = es_client
                    .es_conn
                    .cat()
                    .shards(CatShardsParts::None)
                    .h(fields)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let resp: String = response.text().await?;
            Ok(resp)
        } else {
            let error_message = format!(
                "[Elasticsearch Error][get_cat_shards()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "특정 인덱스에 데이터를 insert 해주는 함수."]
    /// # Arguments
    /// * `index_name`  - 인덱스 이름
    /// * `document`    - 색인할 내용
    ///
    /// # Returns
    /// * Result<(), anyhow::Error>
    async fn post_doc(&self, index_name: &str, document: Value) -> Result<(), anyhow::Error> {
        /* 클로저 내에서 사용할 복사본을 생성 */
        let document_clone = document.clone();

        let response = self
            .execute_on_any_node(|es_client| {
                /* 클로저 내부에서 클론한 값 사용 */
                let value = document_clone.clone();

                async move {
                    let response = es_client
                        .es_conn
                        .index(IndexParts::Index(index_name))
                        .body(value)
                        .send()
                        .await?;

                    Ok(response)
                }
            })
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!(
                "[Elasticsearch Error][post_doc()] Failed to index document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 클러스터의 이름을 가져와주는 함수."]
    fn get_cluster_name(&self) -> String {
        self.cluster_name().to_string()
    }

    #[doc = "Cluster 내의 모든 호스트들을 반환해주는 함수."]
    fn get_cluster_all_host_infos(&self) -> Vec<String> {
        let mut hosts: Vec<String> = Vec::new();

        self.es_clients.iter().for_each(|es_client| {
            hosts.push(es_client.host.clone());
        });

        hosts
    }

    #[doc = "Cluster 정보를 맵핑해줄 index pattern 형식을 반환."]
    fn get_cluster_index_pattern(&self) -> String {
        self.index_pattern.to_string()
    }
}
