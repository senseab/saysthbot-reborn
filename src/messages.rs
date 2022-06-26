pub const BOT_TEXT_MESSAGE_ONLY: &'static str = "仅支持文本信息";
pub const BOT_TEXT_FORWARDED_ONLY: &'static str = "仅支持转发信息";
pub const BOT_TEXT_USER_ONLY: &'static str = "仅支持用户信息";
pub const BOT_TEXT_NO_BOT: &'static str = "不支持 bot 消息";
pub const BOT_TEXT_NOTED: &'static str = "✅ `{data}` 已记录";
pub const BOT_TEXT_NOTICE: &'static str = "[{username}](tg://user?id={user_id}) 转发了你的 `{data}`\n* 你可以使用 /list 命令查看自己或者他人被记录的信息\n* 你可以使用 /del 命令删除某条自己的信息\n* 你也可以使用 /mute 或者 /unmute 命令开启或者关闭提醒";
pub const BOT_TEXT_WELCOME: &'static str =
    "✅ 注册成功！如果有别人记录了你的消息，这里会有提醒，可使用 /mute 命令关闭提醒";
pub const BOT_HELP: &'static str = "**帮助**\n\n* /list `[@username]` 列出已记录的内容\n* /del `id` 删除对应id的记录，只能删除自己的\n* /mute 关闭提醒\n* /unmute 开启提醒";
pub const BOT_ABOUT: &'static str =
    "Say something bot - Reborn\n\n[Github](https://github.com/senseab/saysthbot-reborn) @ssthbot";
pub const BOT_TEXT_MUTE_STATUS: &'static str = "提醒状态：{status}";
pub const BOT_TEXT_STATUS_ON: &'static str = "✅ 开启";
pub const BOT_TEXT_STATUS_OFF: &'static str = "❎ 关闭";
pub const BOT_TEXT_DELETED: &'static str = "已删除";
