use std::collections::HashMap;

use crate::callback_commands::CallbackCommands;
use crate::db_controller::Controller;
use crate::messages::*;
use crate::{commands::CommandHandler, commands::Commands, config::Args};
use migration::DbErr;
use strfmt::Format;

use teloxide::utils::command::BotCommands;
use teloxide::{
    prelude::*, types::ForwardedFrom, types::InlineQueryResult, types::InlineQueryResultArticle,
    types::InputMessageContent, types::InputMessageContentText, types::ParseMode,
    types::ReplyMarkup, types::UpdateKind, RequestError,
};
use wd_log::{log_debug_ln, log_error_ln, log_info_ln, log_panic, log_warn_ln};

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

        self.register_commands().await;

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

    async fn register_commands(&self) {
        if let Err(error) = self
            .bot
            .set_my_commands(Commands::bot_commands())
            .send()
            .await
        {
            self.default_error_handler(&error);
        } else {
            log_info_ln!("commands registered")
        }
    }

    async fn update_handler(&self, update: &Update) {
        match &update.kind {
            UpdateKind::Message(ref message) => self.message_handler(message).await,
            UpdateKind::InlineQuery(inline_query) => self.inline_query_hander(inline_query).await,
            UpdateKind::CallbackQuery(callback) => self.callback_handler(callback).await,
            kind => self.default_update_hander(&kind).await,
        }
    }

    async fn default_update_hander(&self, update_kind: &UpdateKind) {
        log_debug_ln!("non-supported kind {:?}", update_kind);
    }

    async fn callback_handler(&self, callback: &CallbackQuery) {
        log_debug_ln!("callback={:#?}", callback);

        let message = match &callback.message {
            Some(msg) => msg,
            None => return,
        };

        let text = match &callback.data {
            Some(text) => text,
            None => return,
        };

        let bot_username = match self.bot.get_me().send().await {
            Ok(result) => result.username.to_owned(),
            Err(error) => {
                self.default_error_handler(&error);
                return;
            }
        };

        let bot_username = match bot_username {
            Some(b) => b,
            None => return,
        };

        let commands = match CallbackCommands::parse(text, bot_username) {
            Ok(c) => c,
            Err(error) => {
                log_warn_ln!("{}", error);
                return;
            }
        };

        match commands {
            CallbackCommands::Page {
                msg_id: _,
                username,
                page,
            } => {
                let (msg, keyboard) = match CommandHandler::record_msg_genrator(
                    self,
                    message,
                    username.as_str(),
                    page,
                )
                .await
                {
                    Some(d) => d,
                    None => return,
                };

                self.edit_text_reply_with_inline_key(message, message.id, msg.as_str(), keyboard)
                    .await;

                match self.bot.answer_callback_query(&callback.id).send().await {
                    Ok(_) => (),
                    Err(error) => self.default_error_handler(&error),
                }
            }
            CallbackCommands::Default => return,
        }
    }

    async fn inline_query_hander(&self, inline_query: &InlineQuery) {
        let results = match self
            .controller
            .get_records_by_keywords(&inline_query.query)
            .await
        {
            Ok(results) => results,
            Err(error) => {
                self.controller.err_handler(error);
                return;
            }
        };

        let mut r: Vec<InlineQueryResult> = vec![];
        for (record, o_user) in results.current_data.iter() {
            let user = match o_user {
                Some(user) => user,
                None => continue,
            };

            let username = match &user.username {
                Some(username) => username,
                None => continue,
            };

            r.push(InlineQueryResult::Article(InlineQueryResultArticle {
                id: record.id.to_string(),
                title: record.message.to_owned(),
                input_message_content: InputMessageContent::Text(InputMessageContentText {
                    message_text: format!(
                        "*{}*: {}",
                        username.trim_start_matches("@"),
                        record.message
                    ),
                    parse_mode: Some(ParseMode::MarkdownV2),
                    entities: None,
                    disable_web_page_preview: Some(true),
                }),
                reply_markup: None,
                url: None,
                hide_url: None,
                description: Some(format!("By: {}", username)),
                thumb_url: None,
                thumb_width: None,
                thumb_height: None,
            }));
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

    async fn message_handler(&self, message: &Message) {
        if let Some(data) = &message.text() {
            self.text_message_heandler(message, data).await
        } else {
            self.default_message_handler(message).await
        }
    }

    async fn text_message_heandler(&self, message: &Message, data: &str) {
        let forward = match message.forward() {
            Some(forward) => forward,
            None => {
                if data.starts_with("/") {
                    self.command_hanler(message).await;
                } else {
                    self.send_text_reply(message, BOT_TEXT_FORWARDED_ONLY).await;
                }
                return;
            }
        };

        match &forward.from {
            ForwardedFrom::User(user) if !user.is_bot => {
                let username = match &user.username {
                    Some(username) => format!("@{}", username),
                    None => user.first_name.to_owned(),
                };

                if let Err(err) = self
                    .controller
                    .add_record(user.id.0.try_into().unwrap(), &username, data.to_string())
                    .await
                {
                    log_error_ln!("{}", err);
                    return;
                }
                let mut vars = HashMap::new();
                vars.insert("data".to_string(), data);

                self.send_text_reply(message, &BOT_TEXT_NOTED.format(&vars).unwrap())
                    .await;

                let from = match message.from() {
                    Some(from) => from,
                    None => return,
                };

                if from.id == user.id {
                    return;
                }

                if match self
                    .controller
                    .get_user_notify(&user.id.0.try_into().unwrap())
                    .await
                {
                    Ok(notify) => notify,
                    Err(error) => {
                        log_error_ln!("{}", error);
                        return;
                    }
                } {
                    let mut vars = HashMap::new();
                    let user_id = user.id.to_string();
                    let data = data.to_string();

                    vars.insert("username".to_string(), &from.first_name);
                    vars.insert("user_id".to_string(), &user_id);
                    vars.insert("data".to_string(), &data);

                    match self
                        .bot
                        .send_message(user.id, &BOT_TEXT_NOTICE.format(&vars).unwrap())
                        .send()
                        .await
                    {
                        Ok(result) => {
                            log_debug_ln!("message sent {:?}", result)
                        }
                        Err(err) => self.default_error_handler(&err),
                    }
                }
            }
            ForwardedFrom::User(_) => {
                self.send_text_reply(message, BOT_TEXT_NO_BOT).await;
            }
            _ => {
                self.send_text_message(message, BOT_TEXT_USER_ONLY).await;
            }
        }
    }

    async fn command_hanler(&self, message: &Message) {
        let msg = match message.text() {
            Some(msg) => msg,
            None => return,
        };

        let bot_username = match self.bot.get_me().send().await {
            Ok(result) => result.username.to_owned(),
            Err(error) => {
                self.default_error_handler(&error);
                return;
            }
        };

        let bot_username = match bot_username {
            Some(b) => b,
            None => return,
        };

        let commands = match Commands::parse(msg, bot_username) {
            Ok(c) => c,
            Err(error) => {
                log_warn_ln!("{}", error);
                return;
            }
        };

        match commands {
            Commands::Help => CommandHandler::help_handler(&self, message).await,
            Commands::About => CommandHandler::about_handler(&self, message).await,
            Commands::Mute => CommandHandler::notify_handler(&self, message, true).await,
            Commands::Unmute => CommandHandler::notify_handler(&self, message, false).await,
            Commands::List { mut username } => {
                if username == "me" {
                    if let Some(from) = message.from() {
                        if let Some(_username) = &from.username {
                            username = format!("@{}", _username);
                        }
                    }
                }

                if username.starts_with("@") {
                    // always start from page=0
                    CommandHandler::list_handler(&self, message, &username, 0).await;
                } else {
                    self.send_text_reply(message, BOT_TEXT_SHOULD_START_WITH_AT)
                        .await;
                }
            }
            Commands::Del { id } => CommandHandler::del_handler(&self, message, id).await,
            Commands::Start => CommandHandler::setup_handler(&self, message).await,
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

    pub async fn send_text_message(&self, message: &Message, text: &str) -> Option<i32> {
        match &self
            .bot
            .send_message(message.chat.id, text)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(result) => {
                log_debug_ln!("message sent {:?}", result);
                Some(result.id)
            }
            Err(error) => {
                self.default_error_handler(error);
                return None;
            }
        }
    }

    pub async fn send_text_reply(&self, message: &Message, text: &str) -> Option<i32> {
        match &self
            .bot
            .send_message(message.chat.id, text)
            .reply_to_message_id(message.id)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(result) => {
                log_debug_ln!("reply sent {:?}", result);
                Some(result.id)
            }
            Err(error) => {
                self.default_error_handler(error);
                None
            }
        }
    }

    pub async fn edit_text_reply_with_inline_key(
        &self,
        message: &Message,
        msg_id: i32,
        text: &str,
        keyboard: ReplyMarkup,
    ) {
        let keyboard = match keyboard {
            ReplyMarkup::InlineKeyboard(keyboard) => keyboard,
            _ => return,
        };

        match &self
            .bot
            .edit_message_text(message.chat.id, msg_id, text)
            .reply_markup(keyboard)
            .parse_mode(ParseMode::MarkdownV2)
            .send()
            .await
        {
            Ok(result) => log_debug_ln!("reply sent {:?}", result),
            Err(error) => self.default_error_handler(error),
        }
    }
}
