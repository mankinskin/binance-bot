#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]
pub mod binance;
pub mod command;
pub mod error;
pub mod keys;
pub mod subscriptions;
pub mod telegram;
pub mod database;

#[cfg(feature = "actix_server")]
pub mod websocket;
#[cfg(feature = "actix_server")]
pub mod actix_server;
#[cfg(feature = "actix_server")]
pub use actix_server::*;

#[cfg(feature = "tide_server")]
pub mod tide_server;
#[cfg(feature = "tide_server")]
pub use tide_server::*;

#[cfg(all(feature = "actix_server", feature = "tide_server"))]
compile_error!("features [`tide_server`, `actix_server`] are mutually exclusive");

#[cfg(feature = "actix_server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    actix_server::run().await
}
#[cfg(feature = "tide_server")]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    tide_server::run().await
}

pub const CLIENT_PATH: &str = "/home/linusb/git/binance-bot/client";
pub const KEY_PATH: &str = "../keys";

use std::fmt::{
    Formatter,
    Display,
    self,
};
#[derive(Debug, Clone)]
pub struct Error(String);
impl From<String> for Error {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{
        Layer,
        Subscriber,
    },
    layer::SubscriberExt,
};
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
    warn,
    trace,
};
pub fn init_tracing() -> WorkerGuard {
    tracing_log::LogTracer::init().unwrap();
    let file_appender = tracing_appender::rolling::hourly("./logs", "log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = Subscriber::builder()
            .with_env_filter("hyper=error,reqwest=error,h2=error,[]=debug")
            .finish()
            .with(Layer::default().with_writer(file_writer));
    tracing::subscriber::set_global_default(subscriber)
        .expect("Unable to set global tracing subscriber");
    info!("Tracing initialized.");
    info!["Info logs enabled"];
    trace!["Trace logs enabled"];
    debug!["Debug logs enabled"];
    warn!["Warning logs enabled"];
    error!["Error logs enabled"];
    guard
}