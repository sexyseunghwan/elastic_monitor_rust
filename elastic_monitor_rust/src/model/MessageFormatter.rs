use crate::common::*;


#[derive(Clone, Serialize, Deserialize, Debug, new)]
pub struct MessageFormatter {
    pub cluster_name: String,
    pub host: String,
    pub err_subject: String,
    pub err_detail: String
}

impl MessageFormatter {
    
    pub fn transfer_msg(&self) -> String {
        
        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        msg_contents.push_str(format!("[err_subject]\n{}\n\n", self.err_subject).as_str());
        msg_contents.push_str(format!("[err_detail]\n{}\n\n", self.err_detail).as_str());
        msg_contents.push_str(format!("[host]\n{}\n\n", self.host).as_str());
            
        msg_contents
    }

}