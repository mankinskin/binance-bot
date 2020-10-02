#![feature(async_closure)]
#![feature(bool_to_option)]

extern crate serde;
extern crate serde_json;
extern crate openlimits;
extern crate tokio;
extern crate async_std;
extern crate async_h1;
extern crate futures;
extern crate futures_core;
extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate chrono;
extern crate telegram_bot;
extern crate warp;
extern crate tracing;
extern crate tracing_subscriber;
extern crate tracing_appender;
extern crate parallel_stream;
extern crate app_model;

mod shared;
mod server;
pub use server::*;
use server::{
    error::Error,
    telegram::{
        self,
    },
    binance::{
        self,
        Binance,
    },
    model::{
        self,
        Model,
    },
    message_stream,
};
use async_std::{
    sync::{
        MutexGuard,
    },
};
use tracing::{
    debug,
};
use tracing_subscriber::{
    fmt,
    layer::{
        SubscriberExt,
    },
};
use tracing_appender::{
    non_blocking::{
        WorkerGuard,
    },
};

pub async fn binance() -> MutexGuard<'static, Binance> {
    binance::BINANCE.lock().await
}
pub async fn model() -> MutexGuard<'static, Model> {
    model::MODEL.lock().await
}

fn init_tracing() -> WorkerGuard {
    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    tracing::subscriber::set_global_default(
        fmt::Subscriber::builder()
            .with_env_filter("server=debug")
            .finish()
            .with(fmt::Layer::default().with_writer(file_writer))
    ).expect("Unable to set global tracing subscriber");
    debug!("Tracing initialized.");
    guard
}
#[tokio::main]
async fn main() -> Result<(), Error> {
    let _guard = init_tracing();
    binance().await.init().await;
    let (
        _telegram_result,
        _server_result,
        ms_result,
    ) = futures::join! {
        telegram::run(),
        server::listen(),
        message_stream::run(),
    };
    ms_result
}
