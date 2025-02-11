use crate::common::*;

use crate::utils_modules::io_utils::*;

use crate::model::use_case_config::*;
use crate::model::SmtpConfig::*;
use crate::model::TelegramConfig::*;

use crate::env_configuration::env_config::*;

static SERVER_CONFIG: once_lazy<Arc<Config>> =
    once_lazy::new(|| Arc::new(initialize_server_config()));

#[doc = "Function to initialize System configuration information instances"]
pub fn initialize_server_config() -> Config {
    info!("initialize_server_config() START!");

    let system_config: Config = Config::new();
    system_config
}

#[doc = "SMTP config 정보"]
pub fn get_smtp_config_info() -> Arc<SmtpConfig> {
    let smtp_config: &Arc<SmtpConfig> = &SERVER_CONFIG.smtp;
    Arc::clone(smtp_config)
}

#[doc = "Telegram config 정보"]
pub fn get_telegram_config_info() -> Arc<TelegramConfig> {
    let telegram_config: &Arc<TelegramConfig> = &SERVER_CONFIG.telegram;
    Arc::clone(telegram_config)
}

#[doc = "Usecase config 정보"]
pub fn get_usecase_config_info() -> Arc<UseCaseConfig> {
    let usecase_config: &Arc<UseCaseConfig> = &SERVER_CONFIG.usecase;
    Arc::clone(usecase_config)
}

#[derive(Debug)]
pub struct Config {
    pub smtp: Arc<SmtpConfig>,
    pub telegram: Arc<TelegramConfig>,
    pub usecase: Arc<UseCaseConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigNotSafe {
    pub smtp: SmtpConfig,
    pub telegram: TelegramConfig,
    pub usecase: UseCaseConfig,
}

impl Config {
    pub fn new() -> Self {
        let system_config: ConfigNotSafe =
            match read_toml_from_file::<ConfigNotSafe>(&SYSTEM_CONFIG_PATH) {
                Ok(system_config) => system_config,
                Err(e) => {
                    error!(
                        "[Error][main()] Failed to retrieve information 'system_config'. : {:?}",
                        e
                    );
                    panic!(
                        "[Error][main()] Failed to retrieve information 'system_config'. : {:?}",
                        e
                    );
                }
            };

        Config {
            smtp: Arc::new(system_config.smtp),
            telegram: Arc::new(system_config.telegram),
            usecase: Arc::new(system_config.usecase),
        }
    }
}
