use std::collections::HashMap;

use strfmt::Format;
use teloxide::prelude::*;

use crate::{
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

        if !user.is_bot {
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
    }

    pub async fn del_handler(bot_s: &BotServer, message: &Message, id: i64) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if !user.is_bot {
            if let Err(error) = bot_s
                .controller
                .del_record(id, user.id.0.try_into().unwrap())
                .await
            {
                bot_s.controller.err_handler(error);
            }
        }
        bot_s.send_text_reply(message, BOT_TEXT_DELETED).await
    }

    pub async fn list_handler(bot_s: &BotServer, message: &Message, username: &str, page: usize) {
        let user = match message.from() {
            Some(user) => user,
            None => return,
        };

        if !user.is_bot {
            let someone = match bot_s.controller.get_user_by_username(username).await {
                Ok(someone) => someone,
                Err(error) => {
                    bot_s.controller.err_handler(error);
                    return;
                }
            };

            let someone = match someone {
                Some(someone) => someone,
                None => return,
            };

            let data = match bot_s
                .controller
                .get_records_by_userid_with_pagination(someone.id, page)
                .await
            {
                Ok(data) => data,
                Err(error) => {
                    bot_s.controller.err_handler(error);
                    return;
                }
            };

            // TODO: data
        }
    }
}
