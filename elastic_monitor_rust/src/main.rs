/*
Author      : Seunghwan Shin 
Create date : 2024-10-02 
Description : Elasticsearch 클러스터의 문제를 탐색하고 telegram 을 통해 문제를 전달해주는 서비스
    
History     : 2024-10-02 Seunghwan Shin       # first create
              2024-10-07 Seunghwan Shin       # Pending Task 모니터링 항목 추가.
*/ 

mod common;

mod controller;
use controller::main_controller::main_controller;

mod utils_modules;

mod service;
mod model;
mod repository;


#[tokio::main]
async fn main() { main_controller().await; }