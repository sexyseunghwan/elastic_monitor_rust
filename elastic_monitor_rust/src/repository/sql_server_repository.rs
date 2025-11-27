use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::sqlserver::rdb_config::*;

use crate::env_configuration::env_config::*;

use crate::traits::repository::sql_server_repository_trait::*;

#[doc = "전역 Sqlserver Client 인스턴스를 선언"]
static SQL_SERVER_REPO: once_lazy<Arc<SqlServerRepositoryPub>> =
    once_lazy::new(initialize_sqlserver_client);

#[derive(Getters, new)]
#[getset(get = "pub")]
pub struct SqlServerRepositoryPub {
    pub pool: Pool,
}

#[doc = "SQL Server 커넥션 풀 초기화 - 애플리케이션 시작 시 1회만 호출"]
fn initialize_sqlserver_client() -> Arc<SqlServerRepositoryPub> {
    info!("initialize_sqlserver_client() START!");

    /* TOML 로딩 */
    let rdb_config: RdbConfig = read_toml_from_file::<RdbConfig>(&SQL_SERVER_INFO_PATH)
        .unwrap_or_else(|_| {
            let err_msg: &str =
                "[ERROR][initialize_sqlserver_client()] Cannot read RdbConfig object.";
            error!("{}", err_msg);
            panic!("{}", err_msg)
        });

    let conn_str: String = format!(
        "Server={},{};Database={};User Id={};Password={};TrustServerCertificate=true;",
        rdb_config.host(),
        rdb_config.port(),
        rdb_config.db_schema(),
        rdb_config.user_id(),
        rdb_config.user_pw(),
    );

    /* Connection Pool 생성 */
    let pool: deadpool_tiberius::deadpool::managed::Pool<Manager> =
        match Manager::from_ado_string(&conn_str).and_then(|m| {
            /* Sql Server Connection Pool 개수 제한 -> 1개로 충분 */
            m.max_size(1)
                .wait_timeout(std::time::Duration::from_secs(30))
                .pre_recycle_sync(|_conn, _metrics| Ok(()))
                .create_pool()
        }) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    "[ERROR][initialize_sqlserver_client] Failed to create pool: {:?}",
                    e
                );
                panic!("{:?}", e);
            }
        };

    Arc::new(SqlServerRepositoryPub::new(pool))
}

#[doc = "SQL_SERVER_REPO를 Thread-safe 하게 이용하는 함수."]
pub fn get_sql_server_repo() -> Arc<SqlServerRepositoryPub> {
    Arc::clone(&SQL_SERVER_REPO)
}

#[async_trait]
impl SqlServerRepository for SqlServerRepositoryPub {
    #[doc = "SQL Server 아이메일러 관련 프로시저 호출"]
    async fn execute_imailer_procedure(
        &self,
        send_email: &str,
        email_subject: &str,
        email_content: &str,
    ) -> Result<(), anyhow::Error> {
        /* 풀에서 커넥션 가져오기 */
        let pool: &deadpool_tiberius::deadpool::managed::Pool<Manager> = self.pool();
        let mut client: deadpool_tiberius::deadpool::managed::Object<Manager> = pool.get().await?;

        /* 프로시저 호출 */
        let results: Vec<Vec<Row>> = client
            .query(
                r#"
                DECLARE @return_value INT;
                EXEC @return_value = NEWSLETTER.dbo.IM_DMAIL_INFO_INS_TEMPLATE_PROC
                    @GUBUN      = @P1,
                    @SENDNAME   = @P2,
                    @SENDEMAIL  = @P3,
                    @RECVNAME   = @P4,
                    @RECVEMAIL  = @P5,
                    @SUBJECT    = @P6,
                    @CONTENT    = @P7,
                    @QRY        = @P8;
                SELECT @return_value AS return_code;
                "#,
                &[
                    &"ALBA",
                    &"알바천국",
                    &"alba@alba.co.kr",
                    &"",
                    &send_email,
                    &email_subject,
                    &email_content,
                    &"",
                ],
            )
            .await?
            .into_results()
            .await?;

        /* 4. 결과 읽기 */
        if let Some(row) = results.first().and_then(|set| set.first()) {
            let return_code: i32 = row.get("return_code").unwrap_or(0);
            if return_code == 0 {
                error!(
                    "[ERROR][execute_imailer_procedure] proc failure - return_code={}",
                    return_code
                );
            }
        } else {
            error!("[ERROR][execute_imailer_procedure] no return row");
        }

        Ok(())
    }
}
