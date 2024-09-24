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

    
}


impl EsObj {

    
    /*
        
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
            let error_message = format!("[Elasticsearch Error][node_cat_indices_query()] Failed to delete document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        } 
    }
    

}