use crate::common::*;


pub struct MessageFormatter {
    pub cluster_name: String,
    pub host: String,
    pub err_subject: String,
    pub err_detail: String
}

impl MessageFormatter {
    
    pub fn transfer_msg(&self) -> Result<String, anyhow::Error> {
        
        let mut msg_contents: String = String::new();
        msg_contents.push_str(format!("==== Error Alert [{}] ====\n", self.cluster_name).as_str());
        //msg_contents.push_str(format!(""));
        //msg_contents.push_str();
        
        
        Ok(String::from("test"))
    }

}