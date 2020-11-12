use crate::{
    subscriptions,
    telegram,
    command,
};

#[derive(Debug, Clone)]
pub enum Error {
    Telegram(telegram::Error),
    Command(command::Error),
    Subscriptions(subscriptions::Error),
    #[cfg(feature = "actix_server")]
    WebSocket(crate::websocket::Error),
    Server(crate::Error),
    Mpsc(futures::channel::mpsc::SendError),
    Multiple(Vec<Error>),
}
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl ToString for Error {
    fn to_string(&self) -> String {
        format!("{:#?}", self)
    }
}
impl<E: Into<Error>> From<Vec<E>> for Error {
    fn from(errors: Vec<E>) -> Self {
        Self::Multiple(errors.into_iter().map(Into::into).collect())
    }
}
impl From<futures::channel::mpsc::SendError> for Error {
    fn from(err: futures::channel::mpsc::SendError) -> Self {
        Self::Mpsc(err)
    }
}
impl From<crate::Error> for Error {
    fn from(err: crate::Error) -> Self {
        Self::Server(err)
    }
}
impl From<telegram::Error> for Error {
    fn from(err: telegram::Error) -> Self {
        Self::Telegram(err)
    }
}
impl From<subscriptions::Error> for Error {
    fn from(err: subscriptions::Error) -> Self {
        Self::Subscriptions(err)
    }
}
impl From<command::Error> for Error {
    fn from(err: command::Error) -> Self {
        Self::Command(err)
    }
}
#[cfg(feature = "actix_server")]
impl From<crate::websocket::Error> for Error {
    fn from(err: crate::websocket::Error) -> Self {
        Self::WebSocket(err)
    }
}
