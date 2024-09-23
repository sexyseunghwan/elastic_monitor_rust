
mod common;
use crate::common::*;

mod utils_modules;
use crate::utils_modules::logger_utils::*;

#[tokio::main]
async fn main() {
    
    set_global_logger();

    info!("START!");
    println!("START!");
}
