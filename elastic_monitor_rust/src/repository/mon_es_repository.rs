use crate::common::*;

use crate::model::configs::{config::get_mon_es_config_info, mon_elastic_config::*};
use crate::repository::es_repository::EsRepositoryImpl;

#[doc = "Elasticsearch connection pool - 모니터링용 싱글톤"]
static MON_ELASTIC_CONN_SEMAPHORE_POOL: once_lazy<Vec<Arc<EsRepositoryImpl>>> =
    once_lazy::new(|| {
        let mon_es_config: &MonElasticConfig = get_mon_es_config_info();
        let cluster_name: &String = mon_es_config.cluster_name();
        let es_host: &Vec<String> = mon_es_config.hosts();
        let es_id: &String = mon_es_config.es_id();
        let es_pw: &String = mon_es_config.es_pw();
        let pool_cnt: usize = mon_es_config.pool_cnt;
        let index_pattern: &String = mon_es_config.index_pattern();
        let per_index_pattern: &String = mon_es_config.per_index_pattern();
        let urgent_index_pattern: &String = mon_es_config.urgent_index_pattern();
        let err_log_index_pattern: &String = mon_es_config.err_log_index_pattern();

        (0..pool_cnt)
            .map(|_| {
                match EsRepositoryImpl::new(
                    cluster_name,
                    es_host.clone(),
                    es_id,
                    es_pw,
                    Some(index_pattern),
                    Some(per_index_pattern),
                    Some(urgent_index_pattern),
                    Some(err_log_index_pattern),
                ) {
                    Ok(repo) => Arc::new(repo),
                    Err(err) => {
                        error!(
                            "[MON_ELASTIC_CONN_SEMAPHORE_POOL] Failed to create repository: {}",
                            err
                        );
                        panic!("Failed to initialize monitoring connection pool: {}", err);
                    }
                }
            })
            .collect()
    });

#[doc = "세마포어 객체"]
static SEMAPHORE: once_lazy<Arc<Semaphore>> = once_lazy::new(|| {
    let mon_es_config: &MonElasticConfig = get_mon_es_config_info();
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
            .as_slice()
            .choose(&mut rand::rng())
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
