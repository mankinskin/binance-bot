pub mod connection;
pub mod connections;
pub use connections::Connections;

use crate::{
    shared::{
        ClientMessage,
        ServerMessage,
        subscription::PriceSubscription,
    },
};
use async_std::stream::interval;
use connection::Connection;
use futures::{
    channel::mpsc::channel,
    SinkExt,
    StreamExt,
};
use std::convert::TryInto;
use std::time::Duration;
#[allow(unused)]
use tracing::{
    debug,
    error,
};
use warp::ws::WebSocket;
use std::collections::HashMap;
pub use connections::ConnectionClientMessage;

#[derive(Debug, Clone)]
pub struct Error(String);
impl<E: ToString> From<E> for Error {
    fn from(s: E) -> Self {
        Self(s.to_string())
    }
}

pub async fn connection(websocket: WebSocket) {
    let (ws_server_sender, ms_server_receiver) = channel(100); // ClientMessages
    let (ms_client_sender, ws_client_receiver) = channel(100); // ServerMessages
    let id = Connections::add(Connection::new(ms_client_sender, ms_server_receiver)).await;
    // get websocket sink and stream
    let (ws_sink, ws_stream) = websocket.split();
    // forward websocket stream to message sink
    let receiver_handle = tokio::spawn(async {
        ws_stream
            .map(|msg: Result<warp::ws::Message, warp::Error>| msg.map_err(Into::into))
            .forward(ws_server_sender.with(|msg: warp::ws::Message| {
                async { msg.try_into() as Result<ClientMessage, Error> }
            }))
            .await
            .expect("Failed to forward websocket receiving stream")
    });
    if let Ok(()) = ws_client_receiver
        .filter_map(|msg: ServerMessage| async { msg.try_into().map(Ok).ok() })
        .forward(ws_sink)
        .await
    {}
    receiver_handle
        .await
        .expect("Failed to join websocket receiver thread");
    if Connections::contains(id).await {
        Connections::remove(id).await;
    }
}
pub async fn handle_message(id: usize, msg: ClientMessage) -> Result<(), Error> {
    debug!("Websocket message from connection {} {:?}", id, msg);
    let response = match msg {
        ClientMessage::AddPriceSubscription(request) => {
            //debug!("Subscribing to market pair {}", &request.market_pair);
            //let id = crate::subscriptions().await.add_subscription(request.clone()).await?;
            //// TODO interval/timer handles
            //crate::server::interval::set(interval(Duration::from_secs(1)));
            //Some(
            //    ServerMessage::PriceHistory(
            //        crate::subscriptions()
            //            .await
            //            .get_subscription(id)
            //            .await?
            //            .subscription
            //            .latest_price_history()
            //            .await?
            //    )
            //)
            unimplemented!()
        },
        ClientMessage::GetPriceSubscriptionList => {
            //debug!("Getting subscription list");
            //crate::server::interval::set(interval(Duration::from_secs(1)));
            //let list: HashMap<usize, PriceSubscription> =
            //    crate::subscriptions()
            //    .await
            //    .subscriptions.clone()
            //    .into_iter()
            //    .map(|(id, cache)| (id, cache.subscription))
            //    .collect();
            //Some(ServerMessage::SubscriptionList(list))
            unimplemented!()
        },
        ClientMessage::Close => {
            Connections::remove(id).await;
            None
        }
        _ => None,
    };
    if let Some(response) = response {
        Connections::connection(id)
            .await
            .expect(&format!("Connection {} not found!", id))
            .send(response)
            .await?;
    }
    Ok(())
}
pub async fn update() -> Result<(), Error> {
    // TODO send subscription updates for each connection
    //Connections::send_all(ServerMessage::PriceHistory(history)).await;
    Ok(())
}
