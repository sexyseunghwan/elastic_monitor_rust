mod common;
use controller::main_controller::main_controller;

use crate::common::*;

mod controller;

mod service;

mod model;

mod utils_modules;
use crate::utils_modules::logger_utils::*;

#[tokio::main]
async fn main() {
    
    set_global_logger();

    main_controller().await;
    
}
