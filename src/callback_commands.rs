use teloxide::utils::command::BotCommands;

#[derive(PartialEq, Debug, BotCommands)]
#[command(rename = "lowercase", prefix = "!")]
pub enum CallbackCommands {
    #[command(description = "internal command page", parse_with = "split")]
    Page {
        msg_id: i32,
        username: String,
        page: usize,
    },

    #[command(description = "default dummy command")]
    Default,
}

impl Default for CallbackCommands {
    fn default() -> Self {
        CallbackCommands::Default
    }
}
