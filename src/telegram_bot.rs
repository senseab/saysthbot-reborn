use crate::config::Args;
use crate::db_controller::Controller;
use crate::messages::*;
use migration::DbErr;
use teloxide::types::ParseMode;
use teloxide::RequestError;
use teloxide::{prelude::*, types::ForwardedFrom, types::UpdateKind};
use wd_log::{log_debug_ln, log_error_ln, log_info_ln, log_panic};

pub struct BotServer {
    controller: Controller,
    bot: Bot,
}

impl BotServer {
    /// Create new bot
    pub async fn new(config: Args) -> Result<Self, DbErr> {
        Ok(Self {
            bot: Bot::new(config.tgbot_token),
            controller: Controller::new(config.database_uri).await?,
        })
    }

    pub async fn init(&self) -> Result<(), DbErr> {
        self.controller.migrate().await
    }

    /// Run the bot
    pub async fn run(&self) {
        match self.bot.get_me().send().await {
            Ok(result) => log_info_ln!(
                "connect succeed: id={}, botname=\"{}\"",
                result.id,
                result.username()
            ),
            Err(error) => log_panic!("{}", error),
        }

        let mut offset_id = 0;

        loop {
            let updates = match self.bot.get_updates().offset(offset_id).send().await {
                Ok(it) => it,
                _ => continue,
            };
            for update in updates {
                self.update_handler(&update).await;
                offset_id = update.id + 1;
            }
        }
    }

    async fn update_handler(&self, update: &Update) {
        match &update.kind {
            UpdateKind::Message(ref message) => self.message_handler(message).await,
            UpdateKind::InlineQuery(inline_query) => self.inline_query_hander(inline_query).await,
            kind => self.default_update_hander(&kind).await,
        }
    }

    async fn default_update_hander(&self, update_kind: &UpdateKind) {
        log_debug_ln!("non-supported kind {:?}", update_kind);
    }

    async fn inline_query_hander(&self, inline_query: &InlineQuery) {
        log_debug_ln!("inline query: {:?}", inline_query);
    }

    async fn message_handler(&self, message: &Message) {
        if let Some(data) = &message.text() {
            self.text_message_heandler(message, data).await
        } else {
            self.default_message_handler(message).await
        }
    }

    async fn text_message_heandler(&self, message: &Message, data: &str) {
        if let Some(forward) = &message.forward() {
            match &forward.from {
                ForwardedFrom::User(user) if !user.is_bot => {
                    self.send_text_reply(
                        message,
                        format!("`{}` {}", data, BOT_TEXT_NOTED).as_str(),
                    )
                    .await;
                }
                ForwardedFrom::User(_) => {
                    self.send_text_reply(message, BOT_TEXT_NO_BOT).await;
                }
                _ => self.send_text_message(message, BOT_TEXT_USER_ONLY).await,
            }
        } else {
            self.send_text_reply(message, BOT_TEXT_FORWARDED_ONLY).await;
        }
    }

    fn default_error_handler(&self, error: &RequestError) {
        log_error_ln!("{:?}", error);
    }

    async fn default_message_handler(&self, message: &Message) {
        log_debug_ln!(
            "non-spported message {:?} from `{:?}`",
            message.kind,
            message.from()
        );
        self.send_text_reply(message, BOT_TEXT_MESSAGE_ONLY).await;
    }

    async fn send_text_message(&self, message: &Message, text: &str) {
        match &self
            .bot
            .send_message(message.chat.id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(result) => log_debug_ln!("message sent {:?}", result),
            Err(error) => self.default_error_handler(error),
        }
    }

    async fn send_text_reply(&self, message: &Message, text: &str) {
        match &self
            .bot
            .send_message(message.chat.id, text)
            .reply_to_message_id(message.id)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(result) => log_debug_ln!("reply sent {:?}", result),
            Err(error) => self.default_error_handler(error),
        }
    }
}
