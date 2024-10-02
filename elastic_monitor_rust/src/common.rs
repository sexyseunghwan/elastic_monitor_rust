pub use std::{ 
    io::Write, 
    io::BufReader, 
    fs::File,
    sync::{Arc, RwLock},
    future::Future
};


pub use tokio::{
    time::sleep, 
    time::Duration
};

pub use log::{info, error};

pub use flexi_logger::{
    Logger, 
    FileSpec, 
    Criterion, 
    Age, 
    Naming, 
    Cleanup, 
    Record
};

pub use serde::{
    Serialize, 
    Deserialize,
    de::DeserializeOwned
};

pub use serde_json::{Value, from_reader};

pub use elasticsearch::{
    Elasticsearch, 
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    http::Url,
    http::response::Response,
    cat::CatIndicesParts,
    cluster::ClusterHealthParts
};

pub use rand::{
    rngs::StdRng, 
    Rng, 
    SeedableRng,
    seq::SliceRandom
};

pub use anyhow::{Result, anyhow};

pub use getset::Getters;
pub use derive_new::new;

pub use futures::future::join_all;

pub use lazy_static::lazy_static;

pub use async_trait::async_trait;


use crate::model::TeleBot::*;

// 전역 Telebot 인스턴스를 선언
lazy_static! {
    pub static ref TELE_BOT: Arc<RwLock<Option<Telebot>>> = Arc::new(RwLock::new(None));
}
