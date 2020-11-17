use shared::{
    subscriptions::{
        Request,
        PriceSubscription,
    },
};
use crate::{
    websocket::Session,
    subscriptions::{
        cache::{
            actor::SubscriptionCacheActor,
        },
    },
};
#[allow(unused)]
use tracing::{
    debug,
    info,
};
use std::{
    collections::HashMap,
};
use rql::*;

use riker::actors::*;

#[actor(Request)]
#[derive(Debug)]
pub struct SubscriptionsActor {
    session: ActorRef<<Session as Actor>::Msg>,
    actors: Option<HashMap<Id<PriceSubscription>, ActorRef<<SubscriptionCacheActor as Actor>::Msg>>>,
}
impl Actor for SubscriptionsActor {
    type Msg = SubscriptionsActorMsg;
    fn pre_start(&mut self, _ctx: &Context<Self::Msg>) {
        //ctx.run(async move {
        //    self.actors = Some(caches().await
        //        .subscriptions
        //        .iter()
        //        .map(|(id, _)|
        //            (
        //                id.clone(), 
        //                ctx.actor_of_args::<SubscriptionCacheActor, _>(
        //                    &format!("Session({}):Subscription({}):cache_actor", self.session, id),
        //                    (id.clone(), self.session.clone())
        //                ).unwrap()
        //            )
        //        )
        //        .collect());
        //});
    }
    fn recv(&mut self, ctx: &Context<Self::Msg>, msg: Self::Msg, sender: Sender) {
        self.receive(ctx, msg, sender);
    }
}
impl ActorFactoryArgs<ActorRef<<Session as Actor>::Msg>> for SubscriptionsActor {
    fn create_args(session: ActorRef<<Session as Actor>::Msg>) -> Self {
        info!("Creating SubscriptionsActor");
        Self {
            session,
            actors: None,
        }
    }
}
impl Receive<Request> for SubscriptionsActor {
    type Msg = SubscriptionsActorMsg;
    fn receive(&mut self, ctx: &Context<Self::Msg>, _msg: Request, _sender: Sender) {
        let _session = self.session.clone();
        ctx.run(async move {
            //match msg {
            //    Request::GetPriceSubscriptionList => {
            //        info!("Getting subscription list");
            //        let list = Self::get_subscription_list().await;
            //        Some(Response::SubscriptionList(list))
            //    },
            //    Request::AddPriceSubscription(request) => {
            //        info!("Subscribing to market pair {}", &request.market_pair);
            //        let id = Self::add_subscription(request.clone()).await.unwrap();
            //        with_ctx::<Self, _, _>(|act, _ctx| {
            //            act.actors.insert(id.clone(), SubscriptionCacheActor::init(id.clone(), session));
            //        });
            //        Some(Response::SubscriptionAdded(id))
            //    },
            //    Request::Subscription(id, req) => {
            //        let id = id.clone();
            //        info!("Request for Subscription {:#?}", id);
            //        let addr = with_ctx::<Self, _, _>(move |act, _ctx| {
            //            act.actors.get(&id).map(Clone::clone)
            //        });
            //        if let Some(sub) = addr {
            //            sub.send(req.clone()).await.unwrap()
            //        } else {
            //            info!("Subscription {:#?} not found", id);
            //            Some(Response::SubscriptionNotFound(id))
            //        }
            //    }
            //}
        }).unwrap().forget();
    }
}
//impl Handler<Response> for SubscriptionsActor {
//    type Result = ResponseActFuture<Self, ()>;
//    fn handle(
//        &mut self,
//        msg: Response,
//        _ctx: &mut Self::Context,
//    ) -> Self::Result {
//        let session = self.session.clone();
//        async move {
//            session.send(ServerMessage::Subscriptions(msg)).await.unwrap();
//        }.interop_actor_boxed(self)
//    }
//}
