use crate::common::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct HostUrls {
    pub host: String,
    pub url: Url,
}
