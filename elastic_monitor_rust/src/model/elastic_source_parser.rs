use crate::common::*;

#[derive(Debug, Deserialize)]
pub struct SearchHit<T> {
    pub _source: T,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse<T> {
    pub hits: HitsWrapper<T>,
}

#[derive(Debug, Deserialize)]
pub struct HitsWrapper<T> {
    pub hits: Vec<SearchHit<T>>,
}
