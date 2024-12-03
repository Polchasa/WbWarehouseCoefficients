use crate::commands_handlers::*;
use std::error::Error;
use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide_macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Команды:")]
pub enum Command {
    #[command(description = "Help")]
    Help,
    #[command(description = "Авторизация пользователя")]
    Start,
    #[command(description = "Отправляет сообщение всем пользователям.")]
    MsgToAll(String), // Передаём текст сообщения
}

pub async fn answer(bot: Bot, msg: Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let bot_user = bot.get_me().await?;
    let bot_username = bot_user.username().to_string();

    if let Some(text) = msg.text() {
        if let Ok(cmd) = Command::parse(text, &bot_username) {
            match cmd {
                Command::Help => {
                    help_command_handler(bot, msg.chat.id).await?;
                }
                Command::Start => {
                    start_command_handler(bot, &msg).await?;
                }
                Command::MsgToAll(text) => {
                    msg_to_all_command_handler(bot, &msg, text).await?;
                }
            }
        } else {
            text_msg_handler(bot, &msg).await?;
        }
    }
    Ok(())
}
