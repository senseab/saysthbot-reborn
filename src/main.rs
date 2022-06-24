mod bot;
mod config;
mod db_controller;
mod messages;

use bot::Bot;
use clap::Parser;
use config::Args;
use telegram_bot::Error;
use wd_log::{log_debug_ln, set_level, set_prefix, DEBUG, INFO};

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

    let bot = Bot::new(args);

    Ok(bot.run().await)
}
