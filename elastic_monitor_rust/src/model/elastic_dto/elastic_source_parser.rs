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

#[derive(Debug, Deserialize)]
pub struct AggregationResponse<T> {
    pub aggregations: T,
}

#[derive(Debug, Deserialize)]
pub struct DateHistogramAggregation {
    pub buckets: Vec<DateHistogramBucket>,
}

#[derive(Debug, Deserialize)]
pub struct DateHistogramBucket {
    pub key_as_string: Option<String>,
    pub _key: i64,
    pub doc_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ErrorLogsAggregation {
    pub logs_per_time: DateHistogramAggregation,
}
