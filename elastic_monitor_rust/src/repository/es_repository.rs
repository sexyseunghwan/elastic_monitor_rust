use crate::common::*;

#[async_trait]
pub trait EsRepository {
    async fn get_indices_info(&self) -> Result<String, anyhow::Error>;
    async fn get_health_info(&self) -> Result<Value, anyhow::Error>;
}


#[derive(Debug, Getters, Clone, new)]
#[getset(get = "pub")]
pub struct EsObj {
    pub es_host: String,
    pub es_pool: Elasticsearch
}

#[async_trait]
impl EsRepository for EsObj {
    
    /*

    */
    async fn get_indices_info(&self) -> Result<String, anyhow::Error> {
        
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
            
    */
    async fn get_health_info(&self) -> Result<Value, anyhow::Error> {
        
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