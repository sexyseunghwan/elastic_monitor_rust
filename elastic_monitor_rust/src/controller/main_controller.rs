use crate::common::*;
use crate::utils_modules::db_connection_utils::*;

pub async fn main_controller() {

    // DB 커넥션
    initialize_db_clients().await;
    
    //

}