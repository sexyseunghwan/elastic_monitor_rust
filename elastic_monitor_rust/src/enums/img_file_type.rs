pub enum ImgFileType {
    NodeConnErr,
    ClusterStatusErr,
    EmgIndiErr
}

impl ImgFileType {
    pub fn get_name(&self) -> String {
        match self {
            ImgFileType::NodeConnErr => "node_conn_err",
            ImgFileType::ClusterStatusErr => "cluster_status_err",
            ImgFileType::EmgIndiErr => "emg_indi_err",
        }
        .to_string()
    }
}