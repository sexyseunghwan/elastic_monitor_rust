use crate::common::*;

#[derive(Builder, Clone, Serialize, Deserialize, Debug, Getters)]
#[getset(get = "pub", set = "pub")]
pub struct UrgentInfo {
    pub host: String,
    pub network_received: f64,
    pub network_transmitted: f64,
    pub process_count: f64,
    pub recv_dropped_packets: f64,
    pub recv_errors_packet: f64,
    pub send_dropped_packets: f64,
    pub send_errors_packet: f64,
    pub system_cpu_usage: f64,
    pub system_disk_usage: f64,
    pub system_memory_usage: f64,
    pub tcp_close_wait: f64,
    pub tcp_connections: f64,
    pub tcp_established: f64,
    pub tcp_listen: f64,
    pub tcp_timewait: f64,
    pub timestamp: String,
    pub udp_sockets: f64,
}

impl UrgentInfo {
    pub fn get_field_value(&self, field_name: &str) -> Option<f64> {
        match field_name {
            "network_received" => Some(self.network_received),
            "network_transmitted" => Some(self.network_transmitted),
            "process_count" => Some(self.process_count),
            "recv_dropped_packets" => Some(self.recv_dropped_packets),
            "recv_errors_packet" => Some(self.recv_errors_packet),
            "send_dropped_packets" => Some(self.send_dropped_packets),
            "send_errors_packet" => Some(self.send_errors_packet),
            "system_cpu_usage" => Some(self.system_cpu_usage),
            "system_disk_usage" => Some(self.system_disk_usage),
            "system_memory_usage" => Some(self.system_memory_usage),
            "tcp_close_wait" => Some(self.tcp_close_wait),
            "tcp_connections" => Some(self.tcp_connections),
            "tcp_established" => Some(self.tcp_established),
            "tcp_listen" => Some(self.tcp_listen),
            "tcp_timewait" => Some(self.tcp_timewait),
            "udp_sockets" => Some(self.udp_sockets),
            _ => None,
        }
    }
}
