use std::collections::HashMap;

use strfmt::Format;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    types::{InlineKeyboardButtonKind, ReplyMarkup},
    utils::command::{BotCommands, ParseError},
};
use wd_log::log_debug_ln;

use crate::{
    db_controller::PaginatedRecordData,
    messages::{
        BOT_ABOUT, BOT_BUTTON_END, BOT_BUTTON_HEAD, BOT_BUTTON_NEXT, BOT_BUTTON_PREV, BOT_HELP,
        BOT_TEXT_DELETED, BOT_TEXT_LOADING, BOT_TEXT_MUTE_STATUS, BOT_TEXT_STATUS_OFF,
        BOT_TEXT_STATUS_ON, BOT_TEXT_WELCOME,
    },
    telegram_bot::BotServer,
};

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename = "lowercase")]
pub enum Commands {
    #[command(description = "显示帮助信息")]
    Help,

    #[command(description = "关于本 Bot")]
    About,

    #[command(description = "关闭提醒")]
    Mute,

    #[command(description = "开启提醒")]
    Unmute,

    #[command(description = "列出已记录的内容", parse_with = "list_command_parser")]
    List { username: String },

    #[command(description = "删除记录")]
    Del { id: i64 },

    #[command(description = "注册")]
    Setup,
}

impl Default for Commands {
    fn default() -> Self {
        Commands::Help
    }
}

fn list_command_parser(input: String) -> Result<(String,), ParseError> {
    log_debug_ln!(
        "list_command_parse = \"{}\", is empty = {}",
        input,
        input.trim().is_empty()
    );

    let output: String;

    if input.trim().is_empty() {
        output = "me".to_string();
    } else {
        output = input
    }

    Ok((output,))
}

pub struct CommandHandler {}

impl CommandHandler {
    pub async fn about_handler(bot_s: &BotServer, message: &Message) {
        bot_s.send_text_reply(message, BOT_ABOUT).await;
    }

    pub async fn help_handler(bot_s: &BotServer, message: &Message) {
        bot_s.send_text_reply(message, BOT_HELP).await;
    }

    pub async fn notify_handler(bot_s: &BotServer, message: &Message, enabled: bool) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if user.is_bot {
            if let Err(error) = bot_s
                .controller
                .set_user_notify(&user.id.0.try_into().unwrap(), enabled)
                .await
            {
                bot_s.controller.err_handler(error);
            }
            let mut vars = HashMap::new();
            vars.insert(
                "status".to_string(),
                match enabled {
                    true => BOT_TEXT_STATUS_ON,
                    false => BOT_TEXT_STATUS_OFF,
                },
            );
            bot_s
                .send_text_reply(message, &BOT_TEXT_MUTE_STATUS.format(&vars).unwrap())
                .await;
        }
    }

    pub async fn setup_handler(bot_s: &BotServer, message: &Message) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if user.is_bot {
            return;
        }

        let user_id: i64 = user.id.0.try_into().unwrap();
        let username = match user.username.to_owned() {
            Some(username) => format!("@{}", username),
            None => user.first_name.to_owned(),
        };
        if let Err(error) = bot_s.controller.register_user(&user_id, &username).await {
            bot_s.controller.err_handler(error);
        }
        bot_s.send_text_reply(message, BOT_TEXT_WELCOME).await;
    }

    pub async fn del_handler(bot_s: &BotServer, message: &Message, id: i64) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if user.is_bot {
            return;
        }

        if let Err(error) = bot_s
            .controller
            .del_record(id, user.id.0.try_into().unwrap())
            .await
        {
            bot_s.controller.err_handler(error);
        }

        bot_s.send_text_reply(message, BOT_TEXT_DELETED).await;
    }

    pub async fn list_handler(bot_s: &BotServer, message: &Message, username: &str, page: usize) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if user.is_bot {
            return;
        }

        let msg_id = match bot_s.send_text_reply(message, BOT_TEXT_LOADING).await {
            Some(id) => id,
            None => return,
        };

        let (msg, markup) = match Self::record_msg_genrator(bot_s, message, username, page).await {
            Some(d) => d,
            None => return,
        };

        bot_s
            .edit_text_reply_with_inline_key(message, msg_id, msg.as_str(), markup)
            .await;
    }

    pub async fn record_msg_genrator(
        bot_s: &BotServer,
        message: &Message,
        username: &str,
        page: usize,
    ) -> Option<(String, ReplyMarkup)> {
        let someone = match bot_s.controller.get_user_by_username(username).await {
            Ok(someone) => someone,
            Err(error) => {
                bot_s.controller.err_handler(error);
                return None;
            }
        };

        let someone = match someone {
            Some(someone) => someone,
            None => return None,
        };

        let data = match bot_s
            .controller
            .get_records_by_userid_with_pagination(someone.id, page)
            .await
        {
            Ok(data) => data,
            Err(error) => {
                bot_s.controller.err_handler(error);
                return None;
            }
        };

        let paginated_record_data = match data {
            Some(d) => d,
            None => return None,
        };

        Some((
            Self::generate_text_record_msg(&paginated_record_data, page),
            Self::generate_inline_keyboard(
                page,
                paginated_record_data.pages_count,
                username,
                message,
            ),
        ))
    }

    fn generate_inline_keyboard(
        page: usize,
        pages_count: usize,
        username: &str,
        message: &Message,
    ) -> ReplyMarkup {
        let inline_keyboards = match page {
            page if page == 0 && pages_count > 1 => vec![
                InlineKeyboardButton {
                    text: BOT_BUTTON_NEXT.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        page + 1
                    )),
                },
                InlineKeyboardButton {
                    text: BOT_BUTTON_END.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        pages_count - 1
                    )),
                },
            ],
            page if page == 0 && pages_count <= 1 => vec![],
            page if page >= pages_count - 1 => vec![
                InlineKeyboardButton {
                    text: BOT_BUTTON_HEAD.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id, username, 0
                    )),
                },
                InlineKeyboardButton {
                    text: BOT_BUTTON_PREV.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        page - 1
                    )),
                },
            ],
            _ => vec![
                InlineKeyboardButton {
                    text: BOT_BUTTON_HEAD.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id, username, 0
                    )),
                },
                InlineKeyboardButton {
                    text: BOT_BUTTON_PREV.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        page - 1
                    )),
                },
                InlineKeyboardButton {
                    text: BOT_BUTTON_NEXT.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        page + 1
                    )),
                },
                InlineKeyboardButton {
                    text: BOT_BUTTON_END.to_string(),
                    kind: InlineKeyboardButtonKind::CallbackData(format!(
                        "!page {} {} {}",
                        message.id,
                        username,
                        pages_count - 1
                    )),
                },
            ],
        };

        ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup {
            inline_keyboard: vec![inline_keyboards],
        })
    }

    fn generate_text_record_msg(
        paginated_record_data: &PaginatedRecordData,
        page: usize,
    ) -> String {
        let mut msg = String::from("```");
        for (message, _) in paginated_record_data.current_data.iter() {
            msg = format!("{}\n{}\t\t\t\t{}", msg, message.id, message.message);
        }
        msg = format!(
            "{}\n```\n{}/{}",
            msg,
            page + 1,
            paginated_record_data.pages_count
        );

        msg
    }
}
