use crate::common::*;

use crate::model::Indicies::*;

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsHelper {
    cluster_name: String,
    mon_es_pool: Vec<EsObj>
}

#[derive(Debug, Getters, Clone, new)]
#[getset(get = "pub")]
pub struct EsObj {
    es_host: String,
    es_pool: Elasticsearch
}

impl EsHelper {

    /* 
        EsHelper의 생성자 -> Elasticsearch cluster connection 정보객체를 생성해줌.
    */
    pub fn new(cluster_name: &str, hosts: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        
        let mut mon_es_clients: Vec<EsObj> = Vec::new();
    
        for url in hosts {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);

            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            mon_es_clients.push(EsObj::new(url, Elasticsearch::new(transport)));
        }
        
        Ok(EsHelper{cluster_name: cluster_name.to_string(), mon_es_pool: mon_es_clients})
    }
    
    
    /*
        Cluster 내에 존재하는 인덱스들의 정보를 쿼리함.
    */
    pub async fn cluster_cat_indices_query(&self) -> Result<String, anyhow::Error> {
    
        for es_obj in self.mon_es_pool.iter() {

            match es_obj.node_cat_indices_query().await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!("{:?}", err);      
                    continue;
                }
            }   
        }
        
        Err(anyhow!("[Elasticsearch Error][cluster_cat_indices_query()] All Elasticsearch connections failed"))
    }

    
    /*
        Cluster 의 health 체크  
    */
    pub async fn cluster_get_health_query(&self) -> Result<Value, anyhow::Error> {
        
        for es_obj in self.mon_es_pool.iter() {

            match es_obj.node_get_health_query().await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!("{:?}", err);      
                    continue;
                }
            }   
        }
        
        Err(anyhow!("[Elasticsearch Error][cluster_get_helth_query()] All Elasticsearch connections failed"))
    }

    /*
        Cluster 내 각 node 들에 connection 검증
    */
    pub async fn cluster_get_ping_query(&self) -> Vec<(String, bool)> {
        
        let futures = self.mon_es_pool.iter().map(|es_obj| {
            
            let es_host = es_obj.es_host.clone();
            let es_pool = es_obj.es_pool.clone();
            
            // 각 노드의 연결확인을 병렬처리로 수행 하기 위함
            async move {
                
                let response = es_pool.ping().send().await;
                let is_success = match response {
                    Ok(res) if res.status_code().is_success() => true,
                    _ => false,
                };
                
                (es_host, is_success)
            }
        });

        // 모든 노드의 ping 결과를 병렬로 처리.
        join_all(futures).await

    }

    
}


impl EsObj {

    
    /*
        Cluster 내에 존재하는 인덱스들의 정보를 쿼리함.
    */
    pub async fn node_cat_indices_query(&self) -> Result<String, anyhow::Error> {

        let response = self.es_pool
            .cat()
            .indices(CatIndicesParts::None)
            .h(&["health", "status", "index"])
            .send()
            .await?;

        if response.status_code().is_success() {
            let response_body: String = response.text().await?;
            Ok(response_body)
        } else {
            let error_message = format!("[Elasticsearch Error][node_cat_indices_query()] Failed to GET document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        } 
    }
    
    
    /*
        Cluster 의 health 체크  
    */
    pub async fn node_get_health_query(&self) -> Result<Value, anyhow::Error> {

        // _cluster/health 요청
        let response = self.es_pool
            .cluster()
            .health(ClusterHealthParts::None)  
            .send()
            .await?;
        
        if response.status_code().is_success() {
            
            let resp: Value = response.json().await?;
            Ok(resp)

        } else {
            let error_message = format!("[Elasticsearch Error][node_get_health_query()] Failed to GET document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }
    
}