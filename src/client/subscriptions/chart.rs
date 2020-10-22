use components::{
    Component,
    Init,
    Viewable,
};
use crate::{
    shared::{
        subscription::{
            PriceSubscription,
            PriceSubscriptionRequest,
            Request,
        },
        ClientMessage,
    },
    chart::{
        self,
        Chart,
    },
};
use openlimits::model::{
    Candle,
    Interval,
};
use seed::{
    *,
    prelude::*,
};
use tracing::debug;

#[derive(Debug)]
pub struct SubscriptionChart {
    subscription: PriceSubscription,
    chart: Chart,
    pub interval: Interval,
    pub last_candle_update: Option<u64>,
    pub error: Option<String>,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Chart(chart::Msg),
    SetTimeInterval(Interval),
    AppendCandles(Vec<Candle>),
}
impl Init<PriceSubscription> for SubscriptionChart {
    fn init(subscription: PriceSubscription, orders: &mut impl Orders<Msg>) -> Self {
        Self {
            chart: Chart::init((), &mut orders.proxy(Msg::Chart)),
            subscription,
            last_candle_update: None,
            interval: Interval::OneMinute,
            error: None,
        }
    }
}
impl SubscriptionChart {
    fn set_interval_request(&self) -> ClientMessage {
        ClientMessage::Subscriptions(Request::UpdatePriceSubscription(
            PriceSubscriptionRequest {
                market_pair: "SOLBTC".into(),
                interval: Some(self.interval),
            }
        ))
    }
    pub fn append_price_history(&mut self, candles: Vec<Candle>) {
        debug!["appending price history"];
        let new_data: Vec<Candle> = if let Some(timestamp) = self.last_candle_update {
            let new_candles = candles.into_iter().skip_while(|candle| candle.time <= timestamp);
            let count = new_candles.clone().count();
            if count > 0 {
                let candle_plural = if count == 1 { "" } else { "s" };
                debug!("Appending {} new candle{}.", count, candle_plural);
            }
            new_candles.collect()
        } else {
            debug!("Setting {} initial candles.", candles.len());
            candles.into_iter().collect()
        };
        self.chart.append_data(new_data);
        self.chart.update_values();
        self.last_candle_update = self.chart.data.last().map(|candle| candle.time);
    }
    fn interval_selection(&self) -> Node<<Self as Component>::Msg> {
        div![
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneMinute)),
                "1m"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::ThreeMinutes)),
                "3m"
            ],
            button![
                ev(Ev::Click, |_| {
                    Msg::SetTimeInterval(Interval::FifteenMinutes)
                }),
                "15m"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneHour)),
                "1h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::FourHours)),
                "4h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::SixHours)),
                "6h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::TwelveHours)),
                "12h"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneDay)),
                "1d"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::ThreeDays)),
                "3d"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneWeek)),
                "1w"
            ],
            button![
                ev(Ev::Click, |_| Msg::SetTimeInterval(Interval::OneMonth)),
                "1M"
            ],
        ]
    }
}
impl Component for SubscriptionChart {
    type Msg = Msg;
    fn update(&mut self, msg: Msg, orders: &mut impl Orders<Self::Msg>) {
        match msg {
            Msg::SetTimeInterval(interval) => {
                if self.interval != interval {
                    self.interval = interval;
                    self.last_candle_update = None;
                    self.chart.clear();
                }
                orders.notify(self.set_interval_request());
            }
            Msg::AppendCandles(candles) => {
                self.append_price_history(candles);
            }
            Msg::Chart(msg) => {
                self.chart.update(msg, &mut orders.proxy(Msg::Chart));
            }
        }
    }
}
impl Viewable for SubscriptionChart {
    fn view(&self) -> Node<Msg> {
        div![
            format!("{:#?}", self.subscription),
            self.interval_selection(),
            self.chart.view().map_msg(Msg::Chart),
        ]
    }
}
