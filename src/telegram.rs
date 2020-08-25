use crate::shared;
use telegram_bot::{
    Api,
    UpdateKind,
    MessageKind,
    Message,
    CanReplySendMessage,
};
pub use telegram_bot::{
    Error as TelegramError,
    Update as TelegramUpdate,
};
use crate::Error;
use lazy_static::lazy_static;

#[derive(Clone)]
pub struct Telegram {
    api: Api,
}
lazy_static! {
    pub static ref TELEGRAM: Telegram = Telegram::new();
}
fn remove_coloring(text: String) -> String {
    let reg = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    reg.replace_all(&text, "").to_string()
}
impl Telegram {
    pub fn new() -> Self {
        let telegram_key = shared::read_key_file("keys/telegram");
        let api = Api::new(telegram_key);
        Self {
            api,
        }
    }
    pub async fn handle_message(&mut self, message: Message) -> Result<(), Error> {
        match message.kind.clone() {
            MessageKind::Text { data, .. } => {
                let cmd = data;
                println!("<{}>: {}", &message.from.first_name, cmd);
                let output = crate::run_command(cmd).await?;
                let result = self.api.send(message.text_reply(format!(
                    "{}", remove_coloring(output),
                ))).await;
                if let Err(e) = result {
                    self.api.send(message.text_reply(format!(
                        "{:#?}", e,
                    ))).await?;
                }
            },
            _ => {},
        }
        Ok(())
    }
    pub async fn update(&mut self, update: TelegramUpdate) -> Result<(), Error> {
        Ok(
            match update.kind {
                UpdateKind::Message(message) => {
                    self.handle_message(message).await?
                },
                UpdateKind::EditedMessage(_message) => {},
                UpdateKind::ChannelPost(_post) => { },
                UpdateKind::EditedChannelPost(_post) => { },
                UpdateKind::InlineQuery(_query) => { },
                UpdateKind::CallbackQuery(_query) => { },
                UpdateKind::Error(_error) => { },
                UpdateKind::Unknown => { },
            }
        )
    }
}
impl std::ops::Deref for Telegram {
    type Target = Api;
    fn deref(&self) -> &Self::Target {
        &self.api
    }
}
