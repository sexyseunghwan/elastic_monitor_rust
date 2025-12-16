use crate::common::*;

#[doc = "env 헬퍼함수 정의"]
fn get_env_or_panic(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let msg: String = format!("[ENV file read Error] '{}' must be set", key);
            error!("{}", msg);
            panic!("{}", msg);
        }
    }
}

#[doc = "Function to globally initialize the 'ELASTIC_INFO_PATH' variable"]
pub static ELASTIC_INFO_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("ELASTIC_INFO_PATH"));

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("EMAIL_RECEIVER_PATH"));

#[doc = "Function to globally initialize the 'SQL_SERVER_INFO_PATH' variable"]
pub static SQL_SERVER_INFO_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("SQL_SERVER_INFO_PATH"));

#[doc = "Function to globally initialize the 'SYSTEM_CONFIG_PATH' variable"]
pub static SYSTEM_CONFIG_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("SYSTEM_CONFIG_PATH"));

#[doc = "Function to globally initialize the 'HTML_TEMPLATE_PATH' variable"]
pub static HTML_TEMPLATE_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("HTML_TEMPLATE_PATH"));

#[doc = "Function to globally initialize the 'ELASTIC_INDEX_INFO_PATH' variable"]
pub static ELASTIC_INDEX_INFO_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("ELASTIC_INDEX_INFO_PATH"));

#[doc = "Function to globally initialize the 'URGENT_CONFIG_PATH' variable"]
pub static URGENT_CONFIG_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("URGENT_CONFIG_PATH"));

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_DEV_PATH' variable"]
pub static EMAIL_RECEIVER_DEV_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("EMAIL_RECEIVER_DEV_PATH"));

#[doc = "Function to globally initialize the 'MON_ELASTIC_INFO_PATH' variable"]
pub static MON_ELASTIC_INFO_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("MON_ELASTIC_INFO_PATH"));

#[doc = "Function to globally initialize the 'REPORT_HTML_TEMPLATE_PATH' variable"]
pub static REPORT_HTML_TEMPLATE_PATH: once_lazy<String> =
    once_lazy::new(|| get_env_or_panic("REPORT_HTML_TEMPLATE_PATH"));
