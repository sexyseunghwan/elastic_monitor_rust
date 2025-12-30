use crate::common::*;

use crate::model::cluster_dto::cluster_config::*;
use crate::model::elastic_dto::{elastic_source_parser::*, host_urls::*};

use crate::utils_modules::io_utils::*;

use crate::env_configuration::env_config::ELASTIC_INFO_PATH;

use crate::traits::repository::es_repository_trait::*;

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
            config.err_log_index_pattern.as_deref(),
        )?;

        elastic_conn_vec.push(es_helper);
    }

    Ok(elastic_conn_vec)
}

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsRepositoryImpl {
    pub cluster_name: String,
    pub es_client: Elasticsearch,
    pub hosts: Vec<String>,
    pub hosts_url_details: Vec<HostUrls>,
    pub index_pattern: Option<String>,
    pub per_index_pattern: Option<String>, /* deprecated... */
    pub urgent_index_pattern: Option<String>,
    pub err_log_index_pattern: Option<String>,
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
        err_log_index_pattern: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        let mut cluster_urls: Vec<Url> = Vec::new();
        let mut hosts_url_details: Vec<HostUrls> = Vec::new();

        for host in &hosts {
            let parse_url: String = if es_id.is_empty() && es_pw.is_empty() {
                format!("http://{}", host)
            } else {
                let encoded_pw: std::borrow::Cow<'_, str> = urlencoding::encode(es_pw);
                format!("http://{}:{}@{}", es_id, encoded_pw, host)
            };

            let es_cluster_urls: Url = Url::parse(&format!("http://{}", host))?;
            let es_url: Url = Url::parse(&parse_url)?;

            cluster_urls.push(es_cluster_urls);
            hosts_url_details.push(HostUrls::new(host.to_string(), es_url));
        }

        /* Using MultiNodeConnectionPool - Automatic load balancing and failover. */
        /* internet */
        let conn_pool: MultiNodeConnectionPool =
            MultiNodeConnectionPool::round_robin(cluster_urls, None);

        /*
            ***
            If the timeout period is set too short, a timeout will occur during aggregation
            ->
            A timeout of 30 to 60 seconds is recommended.
            ***
        */
        let mut builder: TransportBuilder =
            TransportBuilder::new(conn_pool).timeout(Duration::from_secs(30));

        /* Apply Basic Authentication at the transport level.*/
        if !es_id.is_empty() && !es_pw.is_empty() {
            builder = builder.auth(EsCredentials::Basic(es_id.to_string(), es_pw.to_string()));
        }

        let transport: EsTransport = builder
            .build()
            .map_err(|e| anyhow!("[EsRepositoryImpl->new] {:?}", e))?;
        let es_client: Elasticsearch = Elasticsearch::new(transport);

        Ok(EsRepositoryImpl {
            cluster_name: cluster_name.to_string(),
            es_client,
            hosts,
            hosts_url_details,
            index_pattern: log_index_pattern.map(str::to_string),
            per_index_pattern: per_index_pattern.map(str::to_string),
            urgent_index_pattern: urgent_index_pattern.map(str::to_string),
            err_log_index_pattern: err_log_index_pattern.map(str::to_string),
        })
    }

    #[doc = "Helper function to check the connection status of a single node."]
    /// # Arguments
    /// * `url` - Host address to check
    ///
    /// # Returns
    /// * bool - connection success status
    async fn check_single_node_connection(url: Url) -> bool {
        match Client::builder().timeout(Duration::from_secs(5)).build() {
            Ok(client) => match client.get(url).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            },
            Err(_) => false,
        }
    }
}

#[async_trait]
impl EsRepository for EsRepositoryImpl {
    #[doc = "Elasticsearch 클러스터 내부에 존재하는 인덱스들의 정보를 가져오는 함수"]
    /// # Returns
    /// * Result<String, anyhow::Error> - 클러스터 내에 존재하는 각 인덱스들의 이름 및 health 정보
    async fn get_indices_info(&self) -> Result<String, anyhow::Error> {
        let response: Response = self
            .es_client
            .cat()
            .indices(CatIndicesParts::None)
            .h(&["health", "status", "index"])
            .send()
            .await?;

        if response.status_code().is_success() {
            let response_body: String = response.text().await?;
            Ok(response_body)
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->get_indices_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 클러스터의 Health Check 해주는 함수."]
    async fn get_health_info(&self) -> Result<Value, anyhow::Error> {
        let response: Response = self
            .es_client
            .cluster()
            .health(ClusterHealthParts::None)
            .send()
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->get_health_info()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Elasticsearch 각 노드들이 현재 문제 없이 통신이 되는지 체크해주는 함수."]
    /// # Returns
    /// * Vec<(String, bool)> - 각 호스트별 연결 상태
    async fn get_node_conn_check(&self) -> Vec<(String, bool)> {
        let mut results: Vec<(String, bool)> = Vec::new();

        for host_info in &self.hosts_url_details {
            let is_connected: bool =
                Self::check_single_node_connection(host_info.url.clone()).await;
            results.push((host_info.host.clone(), is_connected));
        }

        results
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

        //왜 비밀번호가 안들어가져있지?
        //info!("es: {:?}", self.es_client);

        let response: Response = self
            .es_client
            .nodes()
            .stats(stats_parts)
            .send()
            .await
            .map_err(|e| anyhow!("[EsRepositoryImpl->get_node_stats] {:?}", e))?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->get_node_stats()] Failed to GET document: Status Code: {}",
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
    async fn get_specific_index_info(&self, index_name: &str) -> anyhow::Result<Value> {
        let response: Response = self
            .es_client
            .indices()
            .stats(IndicesStatsParts::Index(&[index_name]))
            .send()
            .await?;

        if response.status_code().is_success() {
            let resp: Value = response.json().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->get_specific_index_info] Failed to GET document: Status Code: {}",
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
            .es_client
            .cat()
            .shards(CatShardsParts::None)
            .h(fields)
            .send()
            .await?;

        if response.status_code().is_success() {
            let resp: String = response.text().await?;
            Ok(resp)
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->get_cat_shards()] Failed to GET document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "GET /_cat/thread_pool"]
    async fn get_cat_thread_pool(&self) -> Result<String, anyhow::Error> {
        let response: Response = self
            .es_client
            .cat()
            .thread_pool(CatThreadPoolParts::None)
            .send()
            .await?;

        if response.status_code().is_success() {
            let body: String = response.text().await?;
            Ok(body)
        } else {
            let msg: String = format!(
                "[EsRepositoryImpl->get_cat_thread_pool()] Failed to GET thread pool info: Status Code: {}",
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
        let response: Response = self
            .es_client
            .index(IndexParts::Index(index_name))
            .body(document)
            .send()
            .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message: String = format!(
                "[EsRepositoryImpl->post_doc()] Failed to index document: Status Code: {}",
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
            .es_client
            .search(SearchParts::Index(&[index_name]))
            .body(es_query)
            .send()
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
                "[EsRepositoryImpl->get_search_query_dto] response status is failed: {:?}",
                error_body
            ))
        }
    }

    #[doc = "특정 인덱스에서 aggregation 쿼리로 데이터를 가져와주는 함수"]
    /// # Arguments
    /// * `es_query`      - Elasticsearch aggregation 쿼리
    /// * `index_name`    - 인덱스 이름
    ///
    /// # Returns
    /// * Result<T, anyhow::Error> - aggregation 결과를 담은 구조체
    async fn get_agg_query<T>(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> anyhow::Result<Option<T>> 
    where 
        T: for<'de> Deserialize<'de> + Send + 'static + std::default::Default
    {
        let response: Response = self
            .es_client
            .search(SearchParts::Index(&[index_name]))
            .body(es_query)
            .send()
            .await?;

        if response.status_code().is_success() {
            let parsed: AggregationResponse<T> = response.json().await?;
            Ok(parsed.aggregations)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[EsRepositoryImpl->get_agg_query] response status is failed: {:?}",
                error_body
            ))
        }
    }

    #[doc = "특정 인덱스에서 쿼리 조건에 맞는 문서의 개수만 가져오는 함수"]
    /// # Arguments
    /// * `es_query`      - Elasticsearch 쿼리 (query 부분만)
    /// * `index_name`    - 인덱스 이름
    ///
    /// # Returns
    /// * Result<u64, anyhow::Error> - 문서 개수
    async fn get_count_query(&self, es_query: &Value, index_name: &str) -> anyhow::Result<u64> {
        let response: Response = self
            .es_client
            .count(CountParts::Index(&[index_name]))
            .body(es_query)
            .send()
            .await?;

        if response.status_code().is_success() {
            let json_response: Value = response.json().await?;
            let count: u64 = json_response["count"].as_u64().ok_or_else(|| {
                anyhow!("[EsRepositoryImpl->get_count_query] Failed to parse count from response")
            })?;
            Ok(count)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[EsRepositoryImpl->get_count_query] response status is failed: {:?}",
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
        self.hosts.clone()
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

    #[doc = "Function that returns the error log index pattern format among cluster metrics."]
    fn get_cluster_index_error_pattern(&self) -> Option<String> {
        self.err_log_index_pattern.clone()
    }
}
