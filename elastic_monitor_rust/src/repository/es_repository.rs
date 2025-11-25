use crate::common::*;

use crate::model::cluster_dto::cluster_config::*;
use crate::model::elastic_dto::elastic_source_parser::*;

use crate::model::configs::{config::*, mon_elastic_config::*};

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::*;

use crate::traits::es_repository_trait::*;

#[doc = "Elasticsearch connection pool - 모니터링용 싱글톤"]
static MON_ELASTIC_CONN_SEMAPHORE_POOL: once_lazy<Vec<Arc<EsRepositoryImpl>>> = once_lazy::new(
    || {
        let mon_es_config: Arc<MonElasticConfig> = get_mon_es_config_info();
        let cluster_name: &String = mon_es_config.cluster_name();
        let es_host: &Vec<String> = mon_es_config.hosts();
        let es_id: &String = mon_es_config.es_id();
        let es_pw: &String = mon_es_config.es_pw();
        let pool_cnt: usize = mon_es_config.pool_cnt;

        (0..pool_cnt)
        .map(|_| {
            match EsRepositoryImpl::new(cluster_name, es_host.clone(), es_id, es_pw, None, None, None, None) {
                Ok(repo) => Arc::new(repo),
                Err(err) => {
                    error!("[MON_ELASTIC_CONN_SEMAPHORE_POOL] Failed to create repository: {}", err);
                    panic!("Failed to initialize monitoring connection pool: {}", err);
                }
            }
        })
        .collect()
    }
);

#[doc = "세마포어 객체"]
static SEMAPHORE: once_lazy<Arc<Semaphore>> = once_lazy::new(|| {
    let mon_es_config: Arc<MonElasticConfig> = get_mon_es_config_info();
    Arc::new(Semaphore::new(mon_es_config.pool_cnt))
});

#[derive(Debug)]
pub struct ElasticConnGuard {
    client: Arc<EsRepositoryImpl>,
    _permit: OwnedSemaphorePermit, /* drop 시 자동 반환 */
}

impl ElasticConnGuard {
    pub async fn new() -> Result<Self, anyhow::Error> {
        info!(
            "[ElasticConnGuard] Available permits: {}",
            SEMAPHORE.available_permits()
        );
        let permit: OwnedSemaphorePermit = SEMAPHORE.clone().acquire_owned().await?;
        info!("[ElasticConnGuard] Acquired semaphore");

        /* 임의로 하나의 클라이언트를 가져옴 (랜덤 선택 가능) */
        let client: Arc<EsRepositoryImpl> = MON_ELASTIC_CONN_SEMAPHORE_POOL
            .choose(&mut rand::thread_rng())
            .cloned()
            .expect("[Error][EalsticConnGuard -> new] No clients available");

        Ok(Self {
            client,
            _permit: permit, /* Drop 시 자동 반환 */
        })
    }
}

impl Deref for ElasticConnGuard {
    type Target = EsRepositoryImpl;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Drop for ElasticConnGuard {
    fn drop(&mut self) {
        info!("[ElasticConnGuard] permit dropped (semaphore released)");
    }
}

pub async fn get_elastic_guard_conn() -> Result<ElasticConnGuard, anyhow::Error> {
    info!("use elasticsearch connection");
    ElasticConnGuard::new().await
}


// #[doc = "모니터링 역할을 수행해주는 Elasticsearch DB 초기화"]
// pub fn initialize_mon_db_client() -> Result<EsRepositoryPub, anyhow::Error> {
    
//     let cluster_config: ClusterConfig = read_toml_from_file::<ClusterConfig>(&ELASTIC_INFO_PATH)?;

//     let mon_cluster: &ClusterInfo = cluster_config.clusters
//         .get(0)
//         .ok_or_else(|| anyhow!("[ERROR][initialize_mon_db_client] Cluster is empty"))?;

//     let es_helper: EsRepositoryPub = EsRepositoryPub::new(
//         &mon_cluster.cluster_name,
//         mon_cluster.hosts.clone(),
//         &mon_cluster.es_id,
//         &mon_cluster.es_pw,
//         mon_cluster.index_pattern.as_deref(),
//         mon_cluster.per_index_pattern.as_deref(),
//         mon_cluster.urgent_index_pattern.as_deref()
//     )?;

//     Ok(es_helper)
// }


#[doc = "모니터링 대상이 되는 Elasticsearch DB 초기화"]
/// # Returns
/// * Result<Vec<EsRepositoryPub>, anyhow::Error> - 모니터링 할 대상 Elasticsearch 정보 list
pub fn initialize_db_clients() -> Result<Vec<EsRepositoryImpl>, anyhow::Error> {
    let mut elastic_conn_vec: Vec<EsRepositoryImpl> = Vec::new();

    let cluster_config: ClusterConfig = read_toml_from_file::<ClusterConfig>(&ELASTIC_INFO_PATH)?;

    for config in &cluster_config.clusters {
        let es_helper: EsRepositoryImpl = EsRepositoryImpl::new(
            &config.cluster_name,
            config.hosts.clone(),
            &config.es_id,
            &config.es_pw,
            config.index_pattern.as_deref(),
            config.per_index_pattern.as_deref(),
            config.urgent_index_pattern.as_deref(),
            config.err_log_index_pattern.as_deref()
        )?;

        elastic_conn_vec.push(es_helper);
    }

    Ok(elastic_conn_vec)
}

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsRepositoryImpl {
    pub cluster_name: String,
    pub es_clients: Vec<Arc<EsClient>>,
    pub index_pattern: Option<String>,
    pub per_index_pattern: Option<String>,
    pub urgent_index_pattern: Option<String>,
    pub err_log_index_pattern: Option<String>
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch,
}

impl EsRepositoryImpl {
    #[doc = "Elasticsearch connection 생성자"]
    /// # Arguments
    /// * `cluster_name`        - Elasticsearch Cluster 이름
    /// * `hosts`               - Elasticsearch host 주소 벡터
    /// * `es_id`               - Elasticsearch 계정정보 - 아이디
    /// * `es_pw`               - Elasticsearch 계정정보 - 비밀번호
    /// * `log_index_pattern`   - Elasticsearch 의 지표정보를 저장해줄 인덱스 패턴 이름
    /// * `per_index_pattern`   - Elasitcsearch 의 각 인덱스 지표를 저장해줄 인덱스 패턴 이름
    /// * `urgent_index_pattern`- Elasticsearch 에서 긴급하게 모니터링 해야 할 인덱스 패턴
    ///
    /// # Returns
    /// * Result<Self, anyhow::Error>
    pub fn new(
        cluster_name: &str,
        hosts: Vec<String>,
        es_id: &str,
        es_pw: &str,
        log_index_pattern: Option<&str>,
        per_index_pattern: Option<&str>,
        urgent_index_pattern: Option<&str>,
        err_log_index_pattern: Option<&str>
    ) -> Result<Self, anyhow::Error> {
        let mut es_clients: Vec<Arc<EsClient>> = Vec::new();

        for url in hosts {
            let parse_url: String = if es_id.is_empty() && es_pw.is_empty() {
                format!("http://{}", url)
            } else {
                let encoded_pw = urlencoding::encode(es_pw);
                format!("http://{}:{}@{}", es_id, encoded_pw, url)
            };

            let es_url: Url = Url::parse(&parse_url)?;
            let conn_pool: SingleNodeConnectionPool = SingleNodeConnectionPool::new(es_url);
            let transport: EsTransport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5, 0))
                .build()?;

            let elastic_conn: Elasticsearch = Elasticsearch::new(transport);
            let es_client: Arc<EsClient> = Arc::new(EsClient::new(url, elastic_conn));
            es_clients.push(es_client);
        }

        Ok(EsRepositoryImpl {
            cluster_name: cluster_name.to_string(),
            es_clients,
            index_pattern:      log_index_pattern.map(str::to_string),
            per_index_pattern:  per_index_pattern.map(str::to_string),
            urgent_index_pattern: urgent_index_pattern.map(str::to_string),
            err_log_index_pattern: err_log_index_pattern.map(str::to_string)
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
        let mut last_error: Option<anyhow::Error> = None;

        /* StdRng를 사용하여 Send 트레잇 문제 해결 - 랜덤 시드로 생성 */
        let mut rng: StdRng = StdRng::from_entropy();

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
impl EsRepository for EsRepositoryImpl {
    #[doc = "Elasticsearch 클러스터 내부에 존재하는 인덱스들의 정보를 가져오는 함수"]
    /// # Returns
    /// * Result<String, anyhow::Error> - 클러스터 내에 존재하는 각 인덱스들의 이름 및 health 정보
    async fn get_indices_info(&self) -> Result<String, anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
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
            let error_message: String = format!(
                "[Elasticsearch Error][get_indices_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 클러스터의 Health Check 해주는 함수."]
    async fn get_health_info(&self) -> Result<Value, anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                /* _cluster/health 요청 */
                let response: Response = es_client
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
            let error_message: String = format!(
                "[Elasticsearch Error][get_health_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 각 노드들이 현재 문제 없이 통신이 되는지 체크해주는 함수."]
    /// # Returns
    /// * Vec<(String, bool)> -
    async fn get_node_conn_check(&self) -> Vec<(String, bool)> {
        let futures = self.es_clients.iter().map(|es_obj| {
            let es_host: String = es_obj.host.clone();
            let es_pool: Elasticsearch = es_obj.es_conn.clone();

            async move {
                let response = es_pool.ping().send().await;
                let is_success: bool = matches!(response, Ok(res) if res.status_code().is_success());

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
        let stats_parts: NodesStatsParts<'_> = if fields.is_empty() {
            NodesStatsParts::None
        } else {
            NodesStatsParts::Metric(fields)
        };

        let response: Response = self
            .execute_on_any_node(|es_client| {
                let stats_parts = stats_parts.clone();
                async move {
                    /* _nodes/stats 요청 */
                    let response: Response =
                        es_client.es_conn.nodes().stats(stats_parts).send().await?;

                    Ok(response)
                }
            })
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[Elasticsearch Error][get_node_stats()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "특정 인덱스의 stats 정보를 쿼리해주는 함수"]
    /// # Arguments
    /// * `index_name` - 인덱스 이름
    ///
    /// # Returns
    /// * Result<Value, anyhow::Error>
    async fn get_specific_index_info(&self, index_name: &str) -> Result<Value, anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
                    .es_conn
                    .indices()
                    .stats(IndicesStatsParts::Index(&[index_name]))
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[Elasticsearch Error][get_specific_index_info()] Failed to GET document: Status Code: {}",
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
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                /* _nodes/stats 요청 */
                let response: Response = es_client
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
            let error_message: String = format!(
                "[Elasticsearch Error][get_cat_shards()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "GET /_cat/thread_pool"]
    async fn get_cat_thread_pool(&self) -> Result<String, anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
                    .es_conn
                    .cat()
                    .thread_pool(CatThreadPoolParts::None)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let body: String = response.text().await?;
            Ok(body)
        } else {
            let msg: String = format!(
                "[Elasticsearch Error][get_cat_thread_pool()] Failed to GET thread pool info: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(msg))
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
        let document_clone: Value = document.clone();

        let response: Response = self
            .execute_on_any_node(|es_client| {
                /* 클로저 내부에서 클론한 값 사용 */
                let value: Value = document_clone.clone();

                async move {
                    let response: Response = es_client
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
            let error_message: String = format!(
                "[Elasticsearch Error][post_doc()] Failed to index document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "특정 인덱스에서 get 쿼리로 데이터를 가져와주는 함수"]
    /// # Arguments
    /// * `es_query`      - Elasticsearch 쿼리
    /// * `index_name`    - 인덱스 이름
    ///
    /// # Returns
    /// * Result<Value, anyhow::Error>
    async fn get_search_query<T: for<'de> Deserialize<'de> + Send + 'static>(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Vec<T>, anyhow::Error> {
        let response: Response = self
            .execute_on_any_node(|es_client| async move {
                let response: Response = es_client
                    .es_conn
                    .search(SearchParts::Index(&[index_name]))
                    .body(es_query)
                    .send()
                    .await?;

                Ok(response)
            })
            .await?;

        if response.status_code().is_success() {
            let parsed: SearchResponse<T> = response.json().await?;
            let dtos: Vec<T> = parsed
                .hits
                .hits
                .into_iter()
                .map(|hit| hit._source)
                .collect();
            Ok(dtos)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[Elasticsearch Error][get_search_query_dto] response status is failed: {:?}",
                error_body
            ))
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
    fn get_cluster_index_pattern(&self) -> Option<String> {
        self.index_pattern.clone()
    }

    #[doc = "Cluster 내부의 모니터링 대상이 되는 인덱스의 지표를 저장해줄 인덱스패턴 형식을 반환"]
    fn get_cluster_index_monitoring_pattern(&self) -> Option<String> {
        self.per_index_pattern.clone()
    }

    #[doc = "Cluster 지표중 긴급하게 모니터링 해야할 인덱스 패턴 형식을 반환"]
    fn get_cluster_index_urgent_pattern(&self) -> Option<String> {
        self.urgent_index_pattern.clone()
    }
}
