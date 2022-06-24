use crate::config::Args;
use crate::messages::*;
use futures::StreamExt;
use telegram_bot::*;
use wd_log::{log_debug_ln, log_error_ln, log_info_ln, log_panic};

pub struct Bot {
    api: Api,
}

impl Bot {
    /// Create new bot
    pub fn new(config: Args) -> Self {
        Self {
            api: Api::new(config.tgbot_token),
        }
    }

    fn default_error_handler(&self, error: Error) {
        log_error_ln!("{:?}", error);
    }

    async fn send_text_message(&self, message: &Message, text: &str) {
        match self
            .api
            .send(message.text_reply(text).parse_mode(ParseMode::MarkdownV2))
            .await
        {
            Ok(result) => log_debug_ln!("message sent {:?}", result),
            Err(error) => self.default_error_handler(error),
        }
    }

    async fn default_message_handler(&self, message: &Message) {
        log_debug_ln!(
            "non-spported message {:?} from `{}`",
            message.kind,
            message.from.id
        );
        self.send_text_message(message, BOT_TEXT_MESSAGE_ONLY).await;
    }

    async fn text_message_heandler(&self, message: &Message, data: &String) {
        if let Some(forward) = &message.forward {
            match &forward.from {
                ForwardFrom::User { user } if !user.is_bot => {
                    self.send_text_message(message, format!("`{}` Noted\\.", data).as_str())
                        .await;
                }
                ForwardFrom::User { user: _ } => {
                    self.send_text_message(message, BOT_TEXT_NO_BOT).await;
                }
                _ => self.send_text_message(message, BOT_TEXT_USER_ONLY).await,
            }
        } else {
            self.send_text_message(message, BOT_TEXT_FORWARDED_ONLY)
                .await
        }
    }

    async fn message_handler(&self, message: &Message) {
        match &message.kind {
            MessageKind::Text { data, .. } => self.text_message_heandler(message, data).await,
            _ => self.default_message_handler(message).await,
        }
    }

    async fn inline_query_hander(&self, inline_query: &InlineQuery) {
        log_debug_ln!("inline query: {:?}", inline_query);
    }

    async fn default_update_hander(&self, update_kind: &UpdateKind) {
        log_debug_ln!("non-supported kind {:?}", update_kind);
    }

    async fn update_handler(&self, update: &Update) {
        match &update.kind {
            UpdateKind::Message(ref message) => self.message_handler(message).await,
            UpdateKind::InlineQuery(ref inline_query) => {
                self.inline_query_hander(inline_query).await
            }
            kind => self.default_update_hander(&kind).await,
        }
    }

    /// Run the bot
    pub async fn run(&self) {
        match self.api.send(GetMe).await {
            Ok(result) => log_info_ln!("connect succeed: {:}", result.id),
            Err(error) => log_panic!("{}", error),
        }

        let mut stream = self.api.stream();

        loop {
            let update = match stream.next().await {
                Some(it) => it,
                _ => continue,
            };
            if let Ok(update) = update {
                self.update_handler(&update).await;
            }
        }
    }
}
