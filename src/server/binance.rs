use async_std::sync::MutexGuard;
use crate::shared::PriceHistoryRequest;
use crate::{
    server::keys,
};
use async_std::sync::{
    Arc,
    Mutex,
};
use lazy_static::lazy_static;
use openlimits::{
    binance::Binance as Api,
    errors::OpenLimitError,
    exchange::OpenLimits,
    model::{
        Candle,
        GetHistoricRatesRequest,
        GetPriceTickerRequest,
        Interval,
        Ticker,
        Paginator,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
#[derive(Clone, Debug)]
pub struct Error(String);

impl ToString for Error {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
impl From<String> for Error {
    fn from(err: String) -> Self {
        Self(err)
    }
}

lazy_static! {
    pub static ref BINANCE: Arc<Mutex<Binance>> = Arc::new(Mutex::new(Binance::new()));
}
pub async fn binance() -> MutexGuard<'static, Binance> {
    BINANCE.lock().await
}
#[derive(Serialize, Deserialize)]
pub struct BinanceCredential {
    secret_key: String,
    api_key: String,
}
impl BinanceCredential {
    pub fn new() -> Self {
        Self {
            api_key: keys::read_key_file("keys/binance_api"),
            secret_key: keys::read_key_file("keys/binance_secret"),
        }
    }
}

pub struct Binance {
    api: Option<OpenLimits<Api>>,
}

impl Binance {
    pub fn new() -> Self {
        Self { api: None }
    }
    pub async fn init(&mut self) {
        let credential = BinanceCredential::new();
        let api = Api::with_credential(&credential.api_key, &credential.secret_key, false).await;
        self.api = Some(OpenLimits::new(api));
        //debug!("Initialized Binance API.");
    }
    fn api<'a>(&'a self) -> Result<&'a OpenLimits<Api>, Error> {
        self.api
            .as_ref()
            .ok_or(OpenLimitError::NoApiKeySet().to_string())
            .map_err(Into::into)
    }
    #[allow(unused)]
    fn api_mut<'a>(&'a mut self) -> Result<&'a mut OpenLimits<Api>, Error> {
        self.api
            .as_mut()
            .ok_or(OpenLimitError::NoApiKeySet().to_string())
            .map_err(Into::into)
    }
    pub async fn get_symbol_price(&self, symbol: &str) -> Result<Ticker, Error> {
        //debug!("Requesting symbol price...");
        self.api()?
            .get_price_ticker(&GetPriceTickerRequest {
                market_pair: symbol.to_string().to_uppercase(),
                ..Default::default()
            })
            .await
            .map_err(|e| Error::from(e.to_string()))
    }
    pub async fn symbol_available(&self, symbol: &str) -> bool {
        self.get_symbol_price(symbol).await.is_ok()
    }
    pub async fn get_symbol_price_history(
        &self,
        req: PriceHistoryRequest,
    ) -> Result<Vec<Candle>, Error> {
        //debug!("Requesting symbol price history...");
        self.api()?
            .get_historic_rates(&GetHistoricRatesRequest {
                market_pair: req.market_pair.to_uppercase(),
                interval: req.interval.unwrap_or(Interval::OneMinute),
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
    }
}
