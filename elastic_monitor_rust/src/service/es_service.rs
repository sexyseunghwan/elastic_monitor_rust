use crate::common::*;

use crate::repository::es_repository::*;

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsHelper {
    cluster_name: String,
    mon_es_pool: Vec<EsObj>
}

impl EsHelper {
    
    /* 
        EsHelper의 생성자 -> Elasticsearch cluster connection 정보객체를 생성해줌.
    */
    pub fn new(cluster_name: &str, hosts: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        
        let mut mon_es_clients: Vec<EsObj> = Vec::new();
        
        for url in hosts {
                
            let parse_url = if es_id.is_empty() || es_pw.is_empty() {
                format!("http://{}", url)
            } else {
                format!("http://{}:{}@{}", es_id, es_pw, url)
            };
            
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
    pub async fn get_cluster_indices(&self) -> Result<String, anyhow::Error> {
    
        for es_obj in self.mon_es_pool.iter() {

            match es_obj.get_indices_info().await {
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
    pub async fn get_cluster_health(&self) -> Result<Value, anyhow::Error> {
        
        for es_obj in self.mon_es_pool.iter() {

            match es_obj.get_health_info().await {
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
    pub async fn get_cluster_conn_check(&self) -> Vec<(String, bool)> {
        
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
