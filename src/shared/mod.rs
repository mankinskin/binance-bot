pub mod subscription;
pub use subscription::PriceSubscription;

use serde::{
    Deserialize,
    Serialize,
};

use app_model::market::PriceHistory;
use openlimits::model::{
    Interval,
    Paginator,
};
use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PriceHistoryRequest {
    pub market_pair: String,
    pub interval: Option<Interval>,
    pub paginator: Option<Paginator<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    GetPriceSubscriptionList,
    AddPriceSubscription(PriceHistoryRequest),
    Close,
    Ping,
    Pong,
    Binary(Vec<u8>),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    PriceHistory(PriceHistory),
    SubscriptionList(HashMap<usize, PriceSubscription>),
}
#[cfg(not(target_arch = "wasm32"))]
use {
    crate::server::websocket::Error,
    std::convert::{
        TryFrom,
        TryInto,
    }
};

#[cfg(not(target_arch = "wasm32"))]
impl TryInto<warp::ws::Message> for ServerMessage {
    type Error = Error;
    fn try_into(self) -> Result<warp::ws::Message, Self::Error> {
        Ok(warp::ws::Message::text(
            serde_json::to_string(&self).map_err(|e| Error::from(e.to_string()))?,
        ))
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<warp::ws::Message> for ClientMessage {
    type Error = Error;
    fn try_from(msg: warp::ws::Message) -> Result<Self, Self::Error> {
        if let Ok(text) = msg.to_str() {
            serde_json::de::from_str(text).map_err(Into::into)
        } else {
            if msg.is_close() {
                Ok(ClientMessage::Close)
            } else if msg.is_ping() {
                Ok(ClientMessage::Ping)
            } else if msg.is_pong() {
                Ok(ClientMessage::Pong)
            } else if msg.is_binary() {
                let bytes = msg.as_bytes().to_vec();
                Ok(ClientMessage::Binary(bytes))
            } else {
                Err(Error::from(format!(
                    "Unable to read message: {:#?}",
                    msg
                )))
            }
        }
    }
}
