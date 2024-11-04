use crate::common::*;

#[derive(Debug, new)]
pub struct Indicies {
    pub index_name: String,
    pub health: String,
    pub status: String
}

impl Indicies {
    
    pub fn get_indicies_status(&self) -> String {

        let format = format!(
            "[{}] index status is {}, open status is {}\n",
            self.index_name, self.health, self.status
        );
        
        format
    }   
}
