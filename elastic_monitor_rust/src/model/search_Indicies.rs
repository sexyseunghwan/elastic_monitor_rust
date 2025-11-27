use crate::common::*;

#[derive(Debug, Getters, new)]
#[getset(get = "pub")]
pub struct SearchIndicies {
    pub cluster_name: String,
    pub index_name: String,
    pub health: String,
    pub status: String,
}

impl SearchIndicies {
    pub fn get_indicies_status(&self) -> String {
        let format = format!(
            "[{}] index status is {}, open status is {}\n",
            self.index_name, self.health, self.status
        );

        format
    }
}
