pub use std::io::{ Write, BufReader };
pub use std::fs::File;
pub use std::sync::{Arc, RwLock};

pub use tokio::time::{sleep, Duration};

pub use log::{info, error};

pub use flexi_logger::{Logger, FileSpec, Criterion, Age, Naming, Cleanup, Record};


pub use serde::{Serialize, Deserialize};
pub use serde_json::{Value, from_reader};
pub use serde::de::DeserializeOwned;

pub use elasticsearch::{
    Elasticsearch, http::transport::SingleNodeConnectionPool
};
pub use elasticsearch::http::transport::TransportBuilder;
pub use elasticsearch::http::Url;
pub use elasticsearch::cat::CatIndicesParts;
pub use elasticsearch::cluster::ClusterHealthParts;

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
