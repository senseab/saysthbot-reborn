use clap::Parser;

const DEFAULT_DATABASE: &'static str = "sqlite:///saysthbot.db";
const DEFAULT_API_URL: &'static str = "https://api.telegram.org";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Enable debug mode
    #[clap(short = 'D', long, value_parser, default_value_t = false)]
    pub debug: bool,

    /// Telegram bot token
    #[clap(short, long, value_parser, env = "TGBOT_TOKEN")]
    pub tgbot_token: String,

    /// Database URI
    #[clap(short, long, value_parser, env = "DATABASE_URI", default_value=DEFAULT_DATABASE)]
    pub database_uri: String,

    /// Api Server URL
    #[clap(long, value_parser, env = "API_URL", default_value=DEFAULT_API_URL)]
    pub api_url: String,
}
