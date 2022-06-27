use std::collections::HashMap;

use crate::db_controller::Controller;
use crate::messages::*;
use crate::{commands::CommandHandler, config::Args};
use migration::DbErr;
use strfmt::Format;
use teloxide::{
    prelude::*, types::ForwardedFrom, types::InlineQueryResult, types::InlineQueryResultArticle,
    types::InputMessageContent, types::InputMessageContentText, types::ParseMode,
    types::UpdateKind, RequestError,
};
use wd_log::{log_debug_ln, log_error_ln, log_info_ln, log_panic};

pub struct BotServer {
    pub controller: Controller,
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
        match self
            .controller
            .get_records_by_keywords(&inline_query.query)
            .await
        {
            Ok(results) => {
                let mut r: Vec<InlineQueryResult> = vec![];
                for (record, o_user) in results.current_data.iter() {
                    if let Some(user) = o_user {
                        if let Some(username) = &user.username {
                            r.push(InlineQueryResult::Article(InlineQueryResultArticle {
                                id: record.id.to_string(),
                                title: record.message.to_owned(),
                                input_message_content: InputMessageContent::Text(
                                    InputMessageContentText {
                                        message_text: format!("`{}`: {}", username, record.message),
                                        parse_mode: Some(ParseMode::MarkdownV2),
                                        entities: None,
                                        disable_web_page_preview: Some(true),
                                    },
                                ),
                                reply_markup: None,
                                url: None,
                                hide_url: None,
                                description: Some(format!("By: {}", username)),
                                thumb_url: None,
                                thumb_width: None,
                                thumb_height: None,
                            }));
                        }
                    }
                }

                if let Err(error) = self
                    .bot
                    .answer_inline_query(&inline_query.id, r.into_iter())
                    .send()
                    .await
                {
                    self.default_error_handler(&error);
                }
            }
            Err(error) => self.controller.err_handler(error),
        }
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
                        Some(username) => format!("@{}", username),
                        None => user.first_name.to_owned(),
                    };
                    match self
                        .controller
                        .add_record(user.id.0.try_into().unwrap(), &username, data.to_string())
                        .await
                    {
                        Ok(_) => {
                            let mut vars = HashMap::new();
                            vars.insert("data".to_string(), data);
                            self.send_text_reply(message, &BOT_TEXT_NOTED.format(&vars).unwrap())
                                .await;

                            match message.from() {
                                Some(from) if from.id != user.id => {
                                    match self
                                        .controller
                                        .get_user_notify(&user.id.0.try_into().unwrap())
                                        .await
                                    {
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
                "about" => CommandHandler::about_handler(&self, message).await,
                "list" => {
                    let mut username = commands[1].trim();
                    if username == "" {
                        if let Some(from) = message.from() {
                            if let Some(_username) = &from.username {
                                username = _username;
                            }
                        }
                    }
                    if username.starts_with("@") {
                        CommandHandler::list_handler(&self, message, username).await;
                    } else {
                        self.send_text_reply(message, BOT_TEXT_SHOULD_START_WITH_AT)
                            .await;
                    }
                }
                "del" => {
                    if let Ok(id) = commands[1].trim().parse::<i64>() {
                        CommandHandler::del_handler(&self, message, id).await;
                    }
                }
                "mute" => CommandHandler::notify_handler(&self, message, true).await,
                "unmute" => CommandHandler::notify_handler(&self, message, false).await,
                "setup" => CommandHandler::setup_handler(&self, message).await,
                "help" => CommandHandler::help_handler(&self, message).await,
                _ => CommandHandler::help_handler(&self, message).await,
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

    pub async fn send_text_message(&self, message: &Message, text: &str) {
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

    pub async fn send_text_reply(&self, message: &Message, text: &str) {
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
