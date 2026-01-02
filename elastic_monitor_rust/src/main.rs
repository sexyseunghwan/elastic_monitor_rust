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
                                                1) 하드코딩 되어있는 경로들을 모든 .env 파일로 빼서 컴파일 없이도 수정될 수 있도록 코드 변경
                                                2) reqwest::client 를 전역적으로 사용하도록 코드 수정
              2025-04-16 Seunghwan Shin       # [v.1.9.0] 색인서버 모니터링 로직 추가
              2025-05-28 Seunghwan Shin       # [v.1.10.0]
                                                1) 나눗셈 버그 수정
                                                2) refresh, flush, translog 지표도 모니터링 추가
                                                3) Index 별로 모니터링 추가
              2025-06-24 Seunghwan Shin       # [v.1.11.0] thread_pool 지표 모니터링 추가
              2025-07-02 Seunghwan Shin       # [v.1.12.0] TCP CLOSE WAIT값 즉각적으로 모니터링 알람 추가
              2025-07-15 Seunghwan Shin       # [v.1.13.0] off-heap 사용량 체크를 위한 지표 추가
              2025-08-01 Seunghwan Shin       # [v.2.0.0]
                                                1) 기존 smtp 를 걷어내고 i-mailer 를 통해서 메일 보내기 기능 추가
                                                2) 서비스 분리원칙 적용
              2025-08-28 Seunghwan Shin       # [v.2.1.0]
                                                1) 리눅스 호환가능하도록 변경
                                                2) 개발계에서 문제가 생길경우에는 단독 메일만 보내도록 처리
              2025-09-11 Seunghwan Shin       # [v.2.2.0] 모니터링 전용 ES 에 메트릭 수집하는 방식으로 코드 변경
              2026-01-02 Seunghwan Shin       # [v.3.0.0] Added the monitoring report feature
*/
mod common;
use common::*;

mod controller;
use controller::main_controller::*;

mod utils_modules;
use utils_modules::logger_utils::*;

mod service;
use service::{
    chart_service::*, metrics_service::*, mon_es_service::*, monitoring_service::*,
    notification_service::*, report_service::*,
};

mod model;

mod repository;
use repository::es_repository::*;

mod env_configuration;

mod traits;

mod enums;

#[tokio::main]
async fn main() {
    /* config 설정 전역 적용 */
    dotenv().ok();

    /* 전역 로거설정 */
    set_global_logger();

    info!("Start Elasticsearch Monitoring Program");

    /* List of Elasticsearch DB connection information for ***monitoring targets*** */
    let es_infos_vec: Vec<EsRepositoryImpl> = initialize_db_clients().unwrap_or_else(|e| {
        error!(
            "[main()] Unable to retrieve 'Elasticsearch' connection information.: {:?}",
            e
        );
        panic!(
            "[main()] Unable to retrieve 'Elasticsearch' connection information.: {:?}",
            e
        )
    });

    let mon_es_infos: EsRepositoryImpl = initialize_mon_db_client().unwrap_or_else(|e| {
        error!(
            "[main()] Unable to retrieve 'Elasticsearch' connection information.: {:?}",
            e
        );
        panic!(
            "[main()] Unable to retrieve 'Elasticsearch' connection information.: {:?}",
            e
        )
    });

    /*
        Shared services (stateless or immutable config)
        These services can be safely shared across all clusters
    */
    let chart_service: Arc<ChartServiceImpl> = Arc::new(ChartServiceImpl::new());
    let notification_service: Arc<NotificationServiceImpl> =
        Arc::new(NotificationServiceImpl::new());
    let mon_es_service: Arc<MonEsServiceImpl<EsRepositoryImpl>> =
        Arc::new(MonEsServiceImpl::new(Arc::new(mon_es_infos)));

    /*
        Handler Dependency Injection(DI)
        Since multiple clusters can be monitored simultaneously,
        dependency injection is performed for each cluster.
    */
    for cluster in es_infos_vec {
        let metric_service: Arc<MetricServiceImpl<EsRepositoryImpl>> =
            Arc::new(MetricServiceImpl::new(Arc::new(RwLock::new(cluster))));

        let monitoring_service: Arc<
            MonitoringServiceImpl<
                MetricServiceImpl<EsRepositoryImpl>,
                NotificationServiceImpl,
                MonEsServiceImpl<EsRepositoryImpl>,
            >,
        > = Arc::new(MonitoringServiceImpl::new(
            Arc::clone(&metric_service),
            Arc::clone(&notification_service),
            Arc::clone(&mon_es_service),
        ));

        let report_service: Arc<
            ReportServiceImpl<
                NotificationServiceImpl,
                ChartServiceImpl,
                MonEsServiceImpl<EsRepositoryImpl>,
            >,
        > = Arc::new(ReportServiceImpl::new(
            Arc::clone(&notification_service),
            Arc::clone(&chart_service),
            Arc::clone(&mon_es_service),
        ));

        let controller: MainController<
            MonitoringServiceImpl<
                MetricServiceImpl<EsRepositoryImpl>,
                NotificationServiceImpl,
                MonEsServiceImpl<EsRepositoryImpl>,
            >,
            ReportServiceImpl<
                NotificationServiceImpl,
                ChartServiceImpl,
                MonEsServiceImpl<EsRepositoryImpl>,
            >,
        > = MainController::new(monitoring_service, report_service);

        tokio::spawn(async move {
            if let Err(e) = controller.main_task().await {
                error!("[main] controller error: {:?}", e);
            }
        });
    }

    if let Err(e) = tokio::signal::ctrl_c().await {
        error!("[main] Failed to listen for Ctrl+C signal: {:?}", e);
    }

    info!("Shutting down...");
}
