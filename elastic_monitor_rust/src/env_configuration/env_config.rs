use crate::common::*;

#[doc = "env 헬퍼함수 정의"]
fn get_env_or_panic(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            let msg = format!("[ENV file read Error] '{}' must be set", key);
            log::error!("{}", msg);
            panic!("{}", msg);
        }
    }
}

#[doc = "Function to globally initialize the 'ELASTIC_INFO_PATH' variable"]
pub static ELASTIC_INFO_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("ELASTIC_INFO_PATH"));

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("EMAIL_RECEIVER_PATH"));

#[doc = "Function to globally initialize the 'SYSTEM_CONFIG_PATH' variable"]
pub static SYSTEM_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("SYSTEM_CONFIG_PATH"));

#[doc = "Function to globally initialize the 'HTML_TEMPLATE_PATH' variable"]
pub static HTML_TEMPLATE_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("HTML_TEMPLATE_PATH"));

#[doc = "Function to globally initialize the 'ELASTIC_INDEX_INFO_PATH' variable"]
pub static ELASTIC_INDEX_INFO_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("ELASTIC_INDEX_INFO_PATH"));

#[doc = "Function to globally initialize the 'URGENT_CONFIG_PATH' variable"]
pub static URGENT_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| get_env_or_panic("URGENT_CONFIG_PATH"));


// #[doc = "Function to globally initialize the 'ELASTIC_INFO_PATH' variable"]
// pub static ELASTIC_INFO_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("ELASTIC_INFO_PATH").unwrap_or_else(|_| {
//         let msg: &str = "[ENV file read Error] 'ELASTIC_INFO_PATH' must be set";
//         error!("{}", msg);
//         panic!("{}", msg);
//     })
// });

// #[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
// pub static EMAIL_RECEIVER_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("EMAIL_RECEIVER_PATH").unwrap_or_else(|_| {
//         let msg: &str = "[ENV file read Error] 'EMAIL_RECEIVER_PATH' must be set";
//         error!("{}", msg);
//         panic!("{}", msg);
//     })
// });

// #[doc = "Function to globally initialize the 'SYSTEM_CONFIG_PATH' variable"]
// pub static SYSTEM_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("SYSTEM_CONFIG_PATH").expect("[ENV file read Error] 'SYSTEM_CONFIG_PATH' must be set")
// });

// #[doc = "Function to globally initialize the 'HTML_TEMPLATE_PATH' variable"]
// pub static HTML_TEMPLATE_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("HTML_TEMPLATE_PATH").expect("[ENV file read Error] 'HTML_TEMPLATE_PATH' must be set")
// });

// #[doc = "Function to globally initialize the 'ELASTIC_INDEX_INFO_PATH' variable"]
// pub static ELASTIC_INDEX_INFO_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("ELASTIC_INDEX_INFO_PATH")
//         .expect("[ENV file read Error] 'ELASTIC_INDEX_INFO_PATH' must be set")
// });

// #[doc = "Function to globally initialize the 'URGENT_CONFIG_PATH' variable"]
// pub static URGENT_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| {
//     env::var("URGENT_CONFIG_PATH").expect("[ENV file read Error] 'URGENT_CONFIG_PATH' must be set")
// });
