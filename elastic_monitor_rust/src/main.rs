/*
Author      : Seunghwan Shin
Create date : 2024-10-02
Description : Elasticsearch 클러스터의 문제를 탐색하고 telegram 을 통해 문제를 전달해주는 서비스

History     : 2024-10-02 Seunghwan Shin       # [v.1.0.0] first create
              2024-10-07 Seunghwan Shin       # [v.1.1.0] Pending Task 모니터링 항목 추가.
              2024-10-08 Seunghwan Shin       # [v.1.1.1] 소스코드에 추상화를 이용해서 아키텍쳐 적용
              2024-10-14 Seunghwan Shin       # [v.1.2.0] Pending Task 모니터링 제외.
              2024-10-17 Seunghwan Shin       # [v.1.3.0]
                                                1) 인덱스 삭제 알고리즘 제거
                                                2) jvm young, old, survivor 지표 모니터링 대상 추가
              2024-10-23 Seunghwan Shin       # [v.1.4.0] Elasticsearch 지표 모니터링 대상 추가
              2024-11-04 Seunghwan Shin       # [v.1.5.0] Elasticsearch cluster 에 문제가 발생한 경우 이메일로도 알람을 받는 기능 추가.
              2024-11-25 Seunghwan Shin       # 1) [v.1.5.1] 설정 json 파일을 toml 파일로 전환
                                              # 2) comment(주석) 정리
              2024-12-04 Seunghwan Shin       # [v.1.6.0] 특정 노드에 연결이 끊기더라도 나머지 노드의 메트릭은 수집하도록 소스 변경
              2025-01-28 Seunghwan Shin       # [v.1.7.0]
                                                1) config 파일 통일 작업
                                                2) Telebot service -> repository 로 바꿈.
                                                3) Tele bot -> Message too long 에러 해결
              2025-02-12 Seunghwan Shin       # [v.1.8.0]
                                                1) 하드코딩 되어있는 경로들을 모둔 .env 파일로 빼서 컴파일 없이도 수정될 수 있도록 코드 변경
                                                2) reqwest::client 를 전역적으로 사용하도록 코두 수정
              2025-04-16 Seunghwan Shin       # [v.1.9.0] 색인서버 모니터링 로직 추가
              2025-05-28 Seunghwan Shin       # [v.1.10.0]
                                                1) 나눗셈 버그 수정
                                                2) refresh, flush, translog 지표도 모니터링 추가
                                                3) Index 별로 모니터링 추가
              2025-06-24 Seunghwan Shin       # [v.1.11.0] thread_pool 지표 모니터링 추가
              2025-07-01 Seunghwan Shin       # [v.1.12.0] TCP CLOSE WAIT값 즉각적으로 모니터링 알람 추가
*/
mod common;
use common::*;

mod handler;
use handler::main_handler::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod service;
use service::metrics_service::*;

mod model;
use model::config::*;
use model::use_case_config::*;

mod repository;
use repository::es_repository::*;

mod env_configuration;

#[tokio::main]
async fn main() {
    dotenv().ok();

    /* 전역 로거설정 */
    set_global_logger();

    info!("Start Elasticsearch Monitoring Program");

    /* Elasticsearch DB 커넥션 정보 */
    let es_infos_vec: Vec<EsRepositoryPub> = match initialize_db_clients() {
        Ok(es_infos_vec) => es_infos_vec,
        Err(e) => {
            error!(
                "[Error][main()] Unable to retrieve 'Elasticsearch' connection information.: {:?}",
                e
            );
            panic!("{:?}", e)
        }
    };

    /*
        Handler 의존주입
        - EsRepositoryPub 를 의존주입하는 이유는 각 Cluster 서버마다 모니터링 대상 Elasticsearch 서버가 다를 수 있기 때문이다.
    */
    let mut handlers: Vec<MainHandler<MetricServicePub<EsRepositoryPub>>> = Vec::new();

    for cluster in es_infos_vec {
        let metirc_service: MetricServicePub<EsRepositoryPub> = MetricServicePub::new(cluster);
        let main_handler: MainHandler<MetricServicePub<EsRepositoryPub>> =
            MainHandler::new(metirc_service);
        handlers.push(main_handler);
    }

    /* 실행환경에 따라 분류 */
    let use_case_binding: Arc<UseCaseConfig> = get_usecase_config_info();
    let use_case: &str = use_case_binding.use_case().as_str();

    /*
        Loop 처리를 통해서 계속 Metric 정보 수집.
    */
    loop {
        /* Async 작업 */
        let futures = handlers.iter().map(|handler| {
            async move {
                handler.task_set().await /* 실제 Task */
            }
        });

        let results: Vec<std::result::Result<(), anyhow::Error>> = join_all(futures).await;

        for result in results {
            match result {
                Ok(_) => {
                    info!("Program processed successfully");
                }
                Err(e) => {
                    error!("[Error][main()] Error processing template: {:?}", e);
                }
            }
        }

        if use_case == "dev" {
            info!("Exit Program");
            break; /* Test code */
        }

        info!("Pending Program...");
        std_sleep(Duration::from_secs(10)); /* 10초 마다 탐색 -> 무한루프가 돌고 있으므로. */
    }
}
