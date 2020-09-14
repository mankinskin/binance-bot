pub mod error;
pub mod telegram;
pub mod keys;
pub mod binance;
pub mod model;
pub mod message_stream;
pub mod command;

use async_std::{
    net::{
        SocketAddr,
    },
};
use futures::{
    StreamExt,
    SinkExt,
};
use warp::{
    Filter,
};
use tracing::{
    debug,
    error,
};
use crate::{
    shared::{
        self,
        ServerMessage,
        ClientMessage,
    },
    Error,
};
use std::convert::TryFrom;
const PKG_PATH: &str = "/home/linusb/git/binance-bot/pkg";
pub async fn websocket_connection(websocket: warp::ws::WebSocket) -> Result<(), Error> {
    debug!("Starting WebSocket connection");
    let (mut ws_tx, mut ws_rx) = websocket.split();
    while let Some(result) = ws_rx.next().await {
        let response = handle_message(result?).await?;
        if let Some(message) = response {
            ws_tx.send(warp::ws::Message::text(serde_json::to_string(&message)?)).await?;
        }
    }
    websocket_closed().await
}
pub async fn handle_message(msg: warp::ws::Message) -> Result<Option<ClientMessage>, Error> {
    debug!("Received websocket msg");
    //debug!("{:#?}", msg);
    Ok(match ServerMessage::try_from(msg)? {
        ServerMessage::GetPriceHistory(req) => {
            Some(ClientMessage::PriceHistory(crate::binance().await.get_symbol_price_history(req).await?))
        },
        _ => None,
    })
}
impl TryFrom<warp::ws::Message> for ServerMessage {
    type Error = Error;
    fn try_from(msg: warp::ws::Message) -> Result<Self, Self::Error> {
        if let Ok(text) = msg.to_str() {
            serde_json::de::from_str(text).map_err(Into::into)
        } else {
            if msg.is_close() {
                Ok(ServerMessage::Close)
            } else if msg.is_ping() {
                Ok(ServerMessage::Ping)
            } else if msg.is_pong() {
                Ok(ServerMessage::Pong)
            } else if msg.is_binary() {
                let bytes = msg.as_bytes().to_vec();
                Ok(ServerMessage::Binary(bytes))
            } else {
                Err(Error::WebSocket(format!("Unable to read message: {:#?}", msg)))
            }
        }
    }
}
pub async fn websocket_closed() -> Result<(), Error> {
    debug!("Closing WebSocket connection");
    Ok(())
}
pub async fn listen() {
    let websocket = warp::path("ws")
                .and(warp::ws())
                .map(|ws: warp::ws::Ws| {
                    ws.on_upgrade(move |ws| async {
                        if let Err(e) = websocket_connection(ws).await {
                            error!("WebSocket error: {:#?}", e);
                        }
                    })
                });
    let api = warp::path("api");
    let price_history = api.and(warp::path("price_history"))
        .and_then(|| async {
            crate::binance().await
                .get_symbol_price_history(shared::PriceHistoryRequest {
                market_pair: "SOLBTC".into(),
                interval: Some(openlimits::model::Interval::OneHour),
                paginator: None,
            })
            .await
            .map(|data| serde_json::to_string(&data).unwrap())
            .map_err(|_err|
                warp::reject::not_found()
            )
        });
    let api_routes = price_history;
    let pkg_dir = warp::fs::dir(PKG_PATH.to_string());
    let logger = warp::log::custom(|info|
        debug!("request from {:?}: {} {} {}ms {}",
            info.remote_addr(),
            info.method(),
            info.path(),
            info.elapsed().as_millis(),
            info.status(),
        )
    );
    let routes = websocket
        .or(api_routes)
        .or(pkg_dir)
        .with(logger);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let server = warp::serve(routes);
    debug!("Starting Server");
    server.run(addr).await
}
