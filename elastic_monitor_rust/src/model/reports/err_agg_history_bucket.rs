use crate::common::*;

use crate::model::elastic_dto::elastic_source_parser::*;

use crate::utils_modules::time_utils::*;

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct ErrorAggHistoryBucket {
    pub cluster_name: String,
    pub date_at: DateTime<Local>,
    pub doc_count: i64,
}

#[doc = "Convert Elasticsearch date histogram buckets to ErrorAggHistoryBucket instances"]
/// # Arguments
/// * `cluster_name` - Name of the cluster
/// * `date_histograms` - Slice of date histogram buckets from Elasticsearch
///
/// # Returns
/// * `Ok(Vec<ErrorAggHistoryBucket>)` - Converted buckets (skips entries without key_as_string)
pub fn convert_from_histogram_bucket(
    cluster_name: &str,
    date_histograms: &[DateHistogramBucket],
) -> anyhow::Result<Vec<ErrorAggHistoryBucket>> {
    let histogram_buckets: Vec<ErrorAggHistoryBucket> = date_histograms
        .iter()
        .filter_map(|bucket| {
            bucket.key_as_string.as_ref().and_then(|date_at_str| {
                /* Convert UTC String to Local DateTime */
                match convert_utc_to_local(date_at_str) {
                    Ok(date_at) => Some(ErrorAggHistoryBucket::new(
                        cluster_name.to_string(),
                        date_at,
                        bucket.doc_count,
                    )),
                    Err(e) => {
                        warn!(
                            "[convert_from_histogram_bucket] Failed to convert timestamp '{}': {:?}",
                            date_at_str, e
                        );
                        None
                    }
                }
            })
        })
        .collect();

    if histogram_buckets.len() < date_histograms.len() {
        warn!(
            "[ErrorAggHistoryBucket::convert_from_histogram_bucket] {} buckets skipped due to missing key_as_string",
            date_histograms.len() - histogram_buckets.len()
        );
    }

    Ok(histogram_buckets)
}
