use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::configs::{
    mon_elastic_config::*, report_config::*, smtp_config::*, telegram_config::*, use_case_config::*,
};

use crate::env_configuration::env_config::*;

static SERVER_CONFIG: once_lazy<Arc<Config>> =
    once_lazy::new(|| Arc::new(initialize_server_config()));

#[doc = "Function to initialize System configuration information instances"]
pub fn initialize_server_config() -> Config {
    info!("initialize_server_config() START!");

    let system_config: Config = Config::new();
    system_config
}

#[doc = "Information of SMTP configuration"]
pub fn get_smtp_config_info() -> &'static SmtpConfig {
    &SERVER_CONFIG.smtp
}

#[doc = "Information of Telegram configuration"]
pub fn get_telegram_config_info() -> &'static TelegramConfig {
    &SERVER_CONFIG.telegram
}

#[doc = "Information of Usecase configuration"]
pub fn get_usecase_config_info() -> &'static UseCaseConfig {
    &SERVER_CONFIG.usecase
}

#[doc = "Information of Elasticsearch configuration"]
pub fn get_mon_es_config_info() -> &'static MonElasticConfig {
    &SERVER_CONFIG.monitor_es
}

#[doc = "Daily report information"]
pub fn get_daily_report_config_info() -> &'static ReportConfig {
    &SERVER_CONFIG.daily_report
}

#[doc = "Weekly report information"]
pub fn get_weekly_report_config_info() -> &'static ReportConfig {
    &SERVER_CONFIG.weekly_report
}

#[doc = "Monthly report information"]
pub fn get_monthly_report_config_info() -> &'static ReportConfig {
    &SERVER_CONFIG.monthly_report
}

#[doc = "Yearly report information"]
pub fn get_yearly_report_config_info() -> &'static ReportConfig {
    &SERVER_CONFIG.yearly_report
}

#[derive(Debug, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Config {
    pub smtp: SmtpConfig,
    pub telegram: TelegramConfig,
    pub usecase: UseCaseConfig,
    pub monitor_es: MonElasticConfig,
    pub daily_report: ReportConfig,
    pub weekly_report: ReportConfig,
    pub monthly_report: ReportConfig,
    pub yearly_report: ReportConfig,
}

impl Config {
    pub fn new() -> Self {
        let system_config: Config = match read_toml_from_file::<Config>(&SYSTEM_CONFIG_PATH) {
            Ok(system_config) => system_config,
            Err(e) => {
                error!(
                    "[Config->new] Failed to retrieve information 'system_config'. : {:?}",
                    e
                );
                panic!(
                    "[Config->new] Failed to retrieve information 'system_config'. : {:?}",
                    e
                );
            }
        };

        Config {
            smtp: system_config.smtp,
            telegram: system_config.telegram,
            usecase: system_config.usecase,
            monitor_es: system_config.monitor_es,
            daily_report: system_config.daily_report,
            weekly_report: system_config.weekly_report,
            monthly_report: system_config.monthly_report,
            yearly_report: system_config.yearly_report,
        }
    }
}
