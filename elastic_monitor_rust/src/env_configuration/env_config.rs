use crate::common::*;

#[doc = "Function to globally initialize the 'ELASTIC_INFO_PATH' variable"]
pub static ELASTIC_INFO_PATH: once_lazy<String> = once_lazy::new(|| {
    env::var("ELASTIC_INFO_PATH").expect("[ENV file read Error] 'ELASTIC_INFO_PATH' must be set")
});

#[doc = "Function to globally initialize the 'EMAIL_RECEIVER_PATH' variable"]
pub static EMAIL_RECEIVER_PATH: once_lazy<String> = once_lazy::new(|| {
    env::var("EMAIL_RECEIVER_PATH")
        .expect("[ENV file read Error] 'EMAIL_RECEIVER_PATH' must be set")
});

#[doc = "Function to globally initialize the 'SYSTEM_CONFIG_PATH' variable"]
pub static SYSTEM_CONFIG_PATH: once_lazy<String> = once_lazy::new(|| {
    env::var("SYSTEM_CONFIG_PATH").expect("[ENV file read Error] 'SYSTEM_CONFIG_PATH' must be set")
});

#[doc = "Function to globally initialize the 'HTML_TEMPLATE_PATH' variable"]
pub static HTML_TEMPLATE_PATH: once_lazy<String> = once_lazy::new(|| {
    env::var("HTML_TEMPLATE_PATH").expect("[ENV file read Error] 'HTML_TEMPLATE_PATH' must be set")
});

#[doc = "Function to globally initialize the 'ELASTIC_INDEX_INFO_PATH' variable"]
pub static ELASTIC_INDEX_INFO_PATH: once_lazy<String> = once_lazy::new(|| {
    env::var("ELASTIC_INDEX_INFO_PATH")
        .expect("[ENV file read Error] 'ELASTIC_INDEX_INFO_PATH' must be set")
});
