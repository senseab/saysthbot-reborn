use std::collections::HashMap;

use crate::config::Args;
use crate::db_controller::Controller;
use crate::messages::*;
use migration::DbErr;
use strfmt::Format;
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
                    let username = match &user.username {
                        Some(username) => username,
                        None => &user.first_name,
                    };
                    match self
                        .controller
                        .add_record(user.id.0, &username, data.to_string())
                        .await
                    {
                        Ok(_) => {
                            let mut vars = HashMap::new();
                            vars.insert("data".to_string(), data);
                            self.send_text_reply(message, &BOT_TEXT_NOTED.format(&vars).unwrap())
                                .await;

                            match message.from() {
                                Some(from) if from.id != user.id => {
                                    match self.controller.get_user_notify(&user.id.0).await {
                                        Ok(notify) if notify => {
                                            let mut vars = HashMap::new();
                                            let user_id = user.id.to_string();
                                            let data = data.to_string();

                                            vars.insert("username".to_string(), &from.first_name);
                                            vars.insert("user_id".to_string(), &user_id);
                                            vars.insert("data".to_string(), &data);

                                            match self
                                                .bot
                                                .send_message(
                                                    user.id,
                                                    &BOT_TEXT_NOTICE.format(&vars).unwrap(),
                                                )
                                                .send()
                                                .await
                                            {
                                                Ok(result) => {
                                                    log_debug_ln!("message sent {:?}", result)
                                                }
                                                Err(err) => self.default_error_handler(&err),
                                            }
                                        }
                                        Ok(_) => (),
                                        Err(err) => log_error_ln!("{}", err),
                                    }
                                }
                                _ => (),
                            }
                        }
                        Err(err) => log_error_ln!("{}", err),
                    }
                }
                ForwardedFrom::User(_) => {
                    self.send_text_reply(message, BOT_TEXT_NO_BOT).await;
                }
                _ => self.send_text_message(message, BOT_TEXT_USER_ONLY).await,
            }
        } else {
            if data.starts_with("/") {
                self.command_hanler(message).await;
            } else {
                self.send_text_reply(message, BOT_TEXT_FORWARDED_ONLY).await;
            }
        }
    }

    async fn command_hanler(&self, message: &Message) {
        if let Some(msg) = message.text() {
            let commands = self.command_spliter(msg);
            match commands[0].as_str() {
                "list" => {}
                "del" => {}
                "mute" => {}
                "unmute" => {}
                "help" => {}
                _ => (),
            }
        }
    }

    fn command_spliter(&self, msg: &str) -> Vec<String> {
        let commands: Vec<&str> = msg.split(" ").collect();
        let command = commands[0].trim_start_matches("/").to_owned();
        let args = commands[1..].join(" ").to_owned();

        vec![command, args]
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
