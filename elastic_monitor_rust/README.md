# Elasticsearch Monitor Rust

Elasticsearch 클러스터를 모니터링하고 문제 발생 시 Telegram 및 이메일로 알림을 전송하는 Rust 기반 모니터링 서비스입니다.

## 📋 주요 기능

- **클러스터 노드 연결 상태 모니터링**: Elasticsearch 노드들의 연결 상태를 실시간으로 확인
- **클러스터 헬스 체크**: 클러스터 상태(GREEN/YELLOW/RED) 모니터링
- **메트릭 수집 및 저장**: 노드 및 인덱스 메트릭을 Elasticsearch에 자동 저장
- **긴급 알람 시스템**: 설정된 임계값 초과 시 즉시 알림 전송
- **다중 알림 채널**: Telegram Bot 및 SMTP 이메일 지원
- **설정 기반 관리**: TOML 파일을 통한 유연한 설정 관리

## 🏗️ 아키텍처

```
src/
├── main.rs                     # 메인 진입점
├── handler/
│   └── main_handler.rs        # 메인 비즈니스 로직 핸들러
├── service/
│   ├── metrics_service.rs     # 메트릭 수집 서비스
│   └── notification_service.rs # 알림 서비스
├── repository/
│   └── es_repository.rs       # Elasticsearch 연결 관리
├── model/                     # 데이터 모델
├── traits/                    # 트레이트 정의
└── utils_modules/             # 유틸리티 모듈
```

## 📦 의존성

주요 라이브러리:
- `tokio`: 비동기 런타임
- `elasticsearch`: Elasticsearch 클라이언트
- `reqwest`: HTTP 클라이언트
- `serde`: 직렬화/역직렬화
- `lettre`: 이메일 전송
- `flexi_logger`: 로깅
- `dotenv`: 환경 변수 관리

## 🚀 설치 및 실행

### 1. 프로젝트 클론
```bash
git clone <repository-url>
cd elastic_monitor_rust
```

### 2. 환경 설정

#### .env 파일 설정
```bash
# .env 파일 예시
HTML_TEMPLATE_PATH="./html/node_info.html"
ELASTIC_INFO_PATH="./config/elastic_server_info.toml"
SYSTEM_CONFIG_PATH="./config/system_config.toml"
EMAIL_RECEIVER_PATH="./config/email_receiver_info.toml"
EMAIL_RECEIVER_DEV_PATH="./config/email_receiver_info_dev.toml"
ELASTIC_INDEX_INFO_PATH="./config/monitoring_index_info.toml"
URGENT_CONFIG_PATH="./config/urgent_index_info.toml"
SQL_SERVER_INFO_PATH="./config/sql_server_info.toml"
```

#### 시스템 설정 (config/system_config.toml)
```toml
[smtp]
smtp_name = "smtp.gmail.com"
credential_id = "your-email@gmail.com"
credential_pw = "your-app-password"

[telegram]
bot_token = "your-telegram-bot-token"
chat_room_id = "your-chat-id"

[usecase]
use_case = "prod"  # "dev" 또는 "prod"

[monitor_es]
cluster_name = "your-cluster-name"
hosts = ["host1:port", "host2:port", "host3:port"]
es_id = "elasticsearch-username"
es_pw = "elasticsearch-password"
pool_cnt = 2
```

#### 모니터링 인덱스 설정 (config/monitoring_index_info.toml)
```toml
[[index]]
cluster_name = "your-cluster-name"
index_name = "index-to-monitor"
```

#### 긴급 알람 설정 (config/urgent_index_info.toml)
```toml
[[urgent]]
metric_name = "tcp_close_wait"
limit = 0
```

### 3. 빌드 및 실행
```bash
# 디버그 빌드
cargo build

# 릴리즈 빌드
cargo build --release

# 실행
cargo run

# 또는 빌드된 바이너리 실행
./target/release/elastic_monitor_rust
```

## 🔧 설정 가이드

### Telegram Bot 설정
1. BotFather에게 `/newbot` 명령으로 새 봇 생성
2. 받은 토큰을 `system_config.toml`의 `bot_token`에 입력
3. 채팅방 ID를 `chat_room_id`에 입력

### SMTP 설정
1. Gmail의 경우 앱 비밀번호 생성 필요
2. `credential_id`에 이메일 주소 입력
3. `credential_pw`에 앱 비밀번호 입력

### Elasticsearch 연결 설정
1. `hosts` 배열에 모든 노드 주소 입력
2. 인증이 필요한 경우 `es_id`, `es_pw` 설정

## 📊 모니터링 항목

### 노드 레벨 모니터링
- 노드 연결 상태
- JVM 메모리 사용량 (Young, Old, Survivor)
- Thread Pool 상태
- TCP 연결 상태 (CLOSE_WAIT)
- Off-heap 메모리 사용량

### 클러스터 레벨 모니터링
- 클러스터 상태 (GREEN/YELLOW/RED)
- 샤드 상태
- 인덱스별 메트릭

### 인덱스 레벨 모니터링
- Refresh, Flush, Translog 지표
- 인덱스별 성능 메트릭

## 📝 로그

로그는 `logs/` 디렉토리에 저장되며, flexi_logger를 사용하여 관리됩니다.

## 🔄 버전 히스토리

- **v2.2.0** (2025-09): 최신 버전
- **v2.1.0** (2025-08): 리눅스 호환성 추가, 개발계 전용 메일 기능
- **v2.0.0** (2025-08): SMTP 개선, 서비스 분리 원칙 적용
- **v1.13.0** (2025-07): Off-heap 사용량 모니터링 추가
- **v1.12.0** (2025-07): TCP CLOSE_WAIT 즉시 알람 추가

## 🛠️ 개발자 정보

- **작성자**: Seunghwan Shin
- **생성일**: 2024-10-02
- **설명**: Elasticsearch 클러스터 문제 탐지 및 알림 서비스

## 📄 라이선스

이 프로젝트는 개인/내부 사용을 위한 모니터링 도구입니다.