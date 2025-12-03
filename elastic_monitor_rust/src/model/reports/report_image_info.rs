use crate::common::*;

#[derive(Debug, Clone, Getters, new)]
#[getset(get = "pub")]
pub struct ReportImageInfo {
    pub index_name: String,
    pub pic_path: PathBuf,
}
