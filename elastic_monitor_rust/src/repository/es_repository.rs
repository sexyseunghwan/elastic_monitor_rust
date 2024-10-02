use crate::common::*;

#[async_trait]
pub trait EsRepository {
    async fn get_indices_info(&self) -> Result<String, anyhow::Error>;
    async fn get_health_info(&self) -> Result<Value, anyhow::Error>;
    async fn get_node_conn_check(&self) -> Vec<(String, bool)>;
}

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsRepositoryPub {
    pub cluster_name: String,
    pub es_clients: Vec<Arc<EsClient>>,
}


#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch
}


impl EsRepositoryPub {
    
    pub fn new(cluster_name: &str, hosts: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {

        let mut es_clients: Vec<Arc<EsClient>> = Vec::new();
        
        for url in hosts {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            let elastic_conn = Elasticsearch::new(transport);
            let es_client = Arc::new(EsClient::new(url, elastic_conn));
            es_clients.push(es_client);
        }

        Ok(EsRepositoryPub{cluster_name: cluster_name.to_string(), es_clients})
    }
    
    
    // Common logic: common node failure handling and node selection
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(Arc<EsClient>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;
    
        // StdRng를 사용하여 Send 트레잇 문제 해결
        let mut rng = StdRng::from_entropy(); // 랜덤 시드로 생성
        
        // 클라이언트 목록을 셔플
        let mut shuffled_clients: Vec<Arc<EsClient>> = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng); // StdRng를 사용하여 셔플
        
        // 셔플된 클라이언트들에 대해 순차적으로 operation 수행
        for es_client in shuffled_clients {
            match operation(es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }
        
        // 모든 노드에서 실패했을 경우 에러 반환
        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }

}

#[async_trait]
impl EsRepository for EsRepositoryPub {
    
    /*
        Elasticsearch 클러스터 내부에 존재하는 인덱스들의 정보를 가져오는 함수
    */
    async fn get_indices_info(&self) -> Result<String, anyhow::Error> {
        
        let response = self.execute_on_any_node(|es_client| async move {
            
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
            let error_message = format!("[Elasticsearch Error][get_indices_info()] Failed to GET document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        } 
    }


    /*
        Elasticsearch 클러스터의 헬스체크를 해주는 함수.
    */
    async fn get_health_info(&self) -> Result<Value, anyhow::Error> {
        
        let response = self.execute_on_any_node(|es_client| async move { 

            // _cluster/health 요청
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
            let error_message = format!("[Elasticsearch Error][get_health_info()] Failed to GET document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }    

    }
    
    
    /*
        Elasticsearch 각 노드들이 현재 문제 없이 통신이 되는지 체크해주는 함수.
    */
    async fn get_node_conn_check(&self) -> Vec<(String, bool)> {

        let futures = self.es_clients.iter().map(|es_obj| {

            let es_host = es_obj.host.clone();
            let es_pool = es_obj.es_conn.clone();

            async move {
                
                let response = es_pool.ping().send().await;
                let is_success = match response {
                    Ok(res) if res.status_code().is_success() => true,
                    _ => false
                };
                
                (es_host, is_success)
            }
        });    
        
        
        join_all(futures).await
    }

}