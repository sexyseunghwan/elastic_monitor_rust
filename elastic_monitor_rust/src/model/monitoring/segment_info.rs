use crate::common::*;

#[derive(Builder, Clone, Serialize, Deserialize, Debug, new)]
pub struct SegmentInfo {
    pub segment_count: u64,
    pub segment_memory_in_byte: u64,
    pub segment_terms_memory_in_bytes: u64,
    pub segment_stored_fields_memory_in_bytes: u64,
    pub segment_term_vectors_memory_in_bytes: u64,
    pub segment_norms_memory_in_byte: u64,
    pub segment_points_memory_in_bytes: u64,
    pub segment_doc_values_memory_in_bytes: u64,
    pub segment_index_writer_memory_in_bytes: u64,
    pub segment_version_map_memory_in_bytes: u64,
    pub segment_fixed_bit_set_memory_in_bytes: u64

}