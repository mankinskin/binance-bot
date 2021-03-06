use crate::{
    keys,
};
use async_std::sync::{
    Arc,
    Mutex,
};
use lazy_static::lazy_static;
use openlimits::{
    binance::Binance as Api,
    exchange::OpenLimits,
    model::{
        GetHistoricRatesRequest,
        GetPriceTickerRequest,
        Interval,
        Ticker,
        Paginator,
    },
};
use app_model::market::PriceHistory;
use serde::{
    Deserialize,
    Serialize,
};
#[allow(unused)]
use tracing::{
    debug,
    info,
    error,
};
use std::fmt::{
    Display,
    Formatter,
    self,
};
#[derive(Clone, Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Self(err)
    }
}

lazy_static! {
    pub static ref BINANCE: Arc<Mutex<Option<Arc<OpenLimits<Api>>>>> = Arc::new(Mutex::new(None));
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
    pub paginator: Option<Paginator<u32>>,
}
#[derive(Serialize, Deserialize)]
pub struct BinanceCredential {
    secret_key: String,
    api_key: String,
}
impl BinanceCredential {
    pub fn new() -> Self {
        Self {
            api_key: keys::read_key_file("binance_api"),
            secret_key: keys::read_key_file("binance_secret"),
        }
    }
}
pub struct Binance;


impl Binance {
    #[cfg(feature = "actix_server")]
    pub async fn init() -> Addr<Self> {
        let credential = BinanceCredential::new();
        let api = Api::with_credential(&credential.api_key, &credential.secret_key, false).await;
        *BINANCE.lock().await = Some(Arc::new(OpenLimits::new(api)));
        //debug!("Initialized Binance API.");
        Self::create(move |_| Binance)
    }
    async fn api<'a>() -> Result<Arc<OpenLimits<Api>>, Error> {
        BINANCE.lock().await
            .as_ref()
            .ok_or(String::from("Binance API not initialized!"))
            .map_err(Into::into)
            .map(Clone::clone)
    }
    pub async fn get_symbol_price(symbol: &str) -> Result<Ticker, Error> {
        //debug!("Requesting symbol price...");
        Self::api().await?
            .get_price_ticker(&GetPriceTickerRequest {
                market_pair: symbol.to_string().to_uppercase(),
                ..Default::default()
            })
            .await
            .map_err(|e| Error::from(e.to_string()))
    }
    pub async fn symbol_available(symbol: &str) -> bool {
        Self::get_symbol_price(symbol).await.is_ok()
    }
    pub async fn get_symbol_price_history(
        req: PriceHistoryRequest,
    ) -> Result<PriceHistory, Error> {
        //info!("Requesting symbol price history",);
        //debug!("{:#?}", req);
        let time_interval = req.interval.unwrap_or(Interval::OneMinute);
        let market_pair = req.market_pair.to_uppercase();
        Self::api().await?
            .get_historic_rates(&GetHistoricRatesRequest {
                market_pair: market_pair.clone(),
                interval: time_interval.clone(),
                paginator: req.paginator.map(|p: Paginator<u32>|
                    Paginator {
                        after: p.after.map(|x| x as u64),
                        before: p.before.map(|x| x as u64),
                        start_time: p.start_time,
                        end_time: p.end_time,
                        limit: p.limit,
                    }
                )
            })
            .await
            .map_err(|e| Error::from(e.to_string()))
            .map(|candles| PriceHistory {
                market_pair,
                time_interval,
                candles,
            })
    }
}

#[cfg(feature = "actix_server")]
use actix::{
    Actor,
    Context,
    Addr,
};
#[cfg(feature = "actix_server")]
use actix_web::ResponseError;

#[cfg(feature = "actix_server")]
impl ResponseError for Error {}
#[cfg(feature = "actix_server")]
impl Actor for Binance {
    type Context = Context<Self>;
}
