mod commands;
mod config;
mod db_controller;
mod messages;
mod telegram_bot;

use clap::Parser;
use config::Args;
use telegram_bot::BotServer;
use wd_log::{log_debug_ln, log_panic, set_level, set_prefix, DEBUG, INFO};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    set_prefix("saysthbot");

    if args.debug {
        set_level(DEBUG);
        log_debug_ln!("{:?}", args);
    } else {
        set_level(INFO);
    }

    let bot = match BotServer::new(args).await {
        Ok(bot) => bot,
        Err(err) => log_panic!("{}", err),
    };

    if let Err(err) = bot.init().await {
        log_panic!("{}", err);
    }

    bot.run().await;
}
