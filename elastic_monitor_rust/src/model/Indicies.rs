use crate::common::*;

#[derive(Serialize, Deserialize, Debug, new)]
pub struct Indicies {
    pub health: String,
    pub status: String,
    pub index: String
}

impl Indicies {

    pub fn get_indicies(&self) -> String {

        let format = format!(
            "[{}] index cluster is {}, open status is {}\n",
            self.index, self.health, self.status
        );
        
        format
    }   
}