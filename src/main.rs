mod config;

use clap::Parser;
use config::Args;
use futures::StreamExt;
use telegram_bot::*;
use wd_log::{log_debug_ln, log_info_ln, log_panic, set_level, set_prefix, DEBUG, INFO};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    set_prefix("saysthbot");

    if args.debug {
        set_level(DEBUG);
        log_debug_ln!("{:?}", args);
    } else {
        set_level(INFO);
    }

    let api = Api::new(args.tgbot_token);
    match api.send(GetMe).await {
        Ok(result) => {
            log_info_ln!("{:?}", result);
        }
        Err(err) => {
            log_panic!("{:?}", err);
        }
    };

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                match message.forward {
                    Some(ref forward) => {
                        if let ForwardFrom::User { ref user } = forward.from {
                            log_debug_ln!(
                                "<{}> ->{}:{}",
                                &message.from.first_name,
                                user.first_name,
                                data
                            );
                            api.send(message.text_reply(format!("`{}` Noted.", data)))
                                .await?;
                        }
                    }
                    None => {
                        log_debug_ln!("<{}>: {}", &message.from.first_name, data);
                        api.send(message.text_reply(format!("Forwarded text message only.")))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
