use std::collections::HashMap;

use strfmt::Format;
use teloxide::prelude::*;

use crate::{
    db_controller::PaginatedRecordData,
    messages::{
        BOT_ABOUT, BOT_HELP, BOT_TEXT_DELETED, BOT_TEXT_MUTE_STATUS, BOT_TEXT_STATUS_OFF,
        BOT_TEXT_STATUS_ON, BOT_TEXT_WELCOME,
    },
    telegram_bot::BotServer,
};

pub struct CommandHandler {}

impl CommandHandler {
    pub async fn about_handler(bot_s: &BotServer, message: &Message) {
        bot_s.send_text_reply(message, BOT_ABOUT).await;
    }

    pub async fn help_handler(bot_s: &BotServer, message: &Message) {
        bot_s.send_text_reply(message, BOT_HELP).await;
    }

    pub async fn notify_handler(bot_s: &BotServer, message: &Message, enabled: bool) {
        if let Some(user) = message.from() {
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
    }

    pub async fn setup_handler(bot_s: &BotServer, message: &Message) {
        if let Some(user) = message.from() {
            if !user.is_bot {
                let user_id: i64 = user.id.0.try_into().unwrap();
                let username = match user.username.to_owned() {
                    Some(username) => username,
                    None => user.first_name.to_owned(),
                };
                if let Err(error) = bot_s.controller.register_user(&user_id, &username).await {
                    bot_s.controller.err_handler(error);
                }
                bot_s.send_text_reply(message, BOT_TEXT_WELCOME).await;
            }
        }
    }

    pub async fn del_handler(bot_s: &BotServer, message: &Message, id: i64) {
        if let Some(user) = message.from() {
            if !user.is_bot {
                if let Err(error) = bot_s.controller.del_record(id, user.id.0.try_into().unwrap()).await {
                    bot_s.controller.err_handler(error);
                }
            }
            bot_s.send_text_reply(message, BOT_TEXT_DELETED).await
        }
    }

    pub async fn list_handler(bot_s: &BotServer, message: &Message, username: &str) {
        if let Some(user) = message.from() {
            if !user.is_bot {
                let data: Option<PaginatedRecordData>;
                match bot_s.controller.get_user_by_username(username).await {
                    Ok(someone) => {
                        if let Some(someone) = someone {
                            data = match bot_s
                                .controller
                                .get_records_by_userid_with_pagination(someone.id, 0)
                                .await
                            {
                                Ok(result) => result,
                                Err(error) => {
                                    bot_s.controller.err_handler(error);
                                    None
                                }
                            };
                        }
                    }
                    Err(error) => bot_s.controller.err_handler(error),
                }

                // TODO: data
            }
        }
    }
}
