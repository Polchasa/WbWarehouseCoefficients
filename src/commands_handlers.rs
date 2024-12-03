use std::error::Error;
use teloxide::prelude::*;
use crate::{api_reauests::*, keyboards::main_menu};

use crate::{database::*, token_decoder::*};
use crate::keyboards::create_warehouse_keyboard;

pub async fn help_command_handler(bot: Bot, id: ChatId) -> Result<(), Box<dyn Error + Send + Sync>> {
    let help_description = "/start - начать пользоваться ботом. (Это дейстиве добавит тебя в список пользователей ботом и так же ты будешь получать уведомления о разных нововведениях)";
    bot.send_message(id, help_description).await?;
    Ok(())
}

pub async fn start_command_handler(bot: Bot, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {   
    let id = UserId(msg.chat.id.0.try_into()?);
    let username = get_username_from_msg(&msg);
    add_user_to_db(id, username).await?; //  добавим юзера в общий список
    set_user_state(id, State::Idle).await?;
    bot.send_message(msg.chat.id, 
        "🍆 Я бот для работы с <b>Wildberris</b>! 🍆\n\nНа <b>Wildberris</b> я могу показать тебе коэффиценты по складам (в скором времени надеюсь смогу уведомлять о 😋вкусных😋 коэффицентах), а так же найду слот с <b>бесплатной или платной приемкой</b> до подходящего коэффицента.\n\nВыбирай!")
    .parse_mode(teloxide::types::ParseMode::Html)
    .reply_markup(main_menu())
    .await?;
    Ok(())
}

pub async fn msg_to_all_command_handler(bot: Bot, msg: &Message, text: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let username: String = get_username_from_msg(&msg);
    let admin_username = "SET_YOUR_LOGIN_HERE".to_string(); // Имя пользователя администратора
    let command_sender_id = msg.chat.id.0;

    if username == admin_username {
        let user_ids = get_user_ids().await?;
        for user in user_ids {
            if user != command_sender_id {
                bot.send_message(ChatId(user), &text).await?;
            }            
        }
        bot.send_message(msg.chat.id, "Отправлено всем.").await?;
        } else {            
            let response_text = format!("Недостатоно прав.");
            bot.send_message(msg.chat.id, response_text).await?;
        }
    Ok(())
}

pub async fn text_msg_handler(bot: Bot, msg: &Message) -> Result<(), Box<dyn Error + Send + Sync>> {
    let id = UserId(msg.chat.id.0.try_into()?);
    let user_state = get_user_state(id).await?;
    if user_state == State::AwaitingToken {
        let token = msg.text().unwrap_or("").to_string(); // Получаем введённый токен
        let is_token_valid = check_token(&token).await?;
        if is_token_valid {
            set_user_state(id, State::Idle).await?;
            set_user_token(id, token.to_string()).await?;   
            fetch_warehouses(&token).await?;
            bot.send_message(msg.chat.id, "Выберите склад")
            .reply_markup(create_warehouse_keyboard(0, 10).await)
            .await?;
        } else {
            if is_token_expired(token).await? {
                bot.send_message(msg.chat.id, "Токен просрочен, введите другой токен").await?;
            } else {
                bot.send_message(msg.chat.id, "Токен невалиден, введите другой токен").await?;
            }
        }
    }
    Ok(())
}

pub async fn bot_started_msg(bot: Bot) -> Result<(), Box<dyn Error + Send + Sync>> {
    let id_i64 = get_id_by_username("polchasaa".to_string()).await?;
    let id = ChatId(id_i64);
    bot.send_message(id, "Бот запущен".to_string()).await?;
    Ok(())
}

fn get_username_from_msg(msg: &Message) -> String {
    match &msg.from {
        Some(user) => match &user.username {
            Some(name) => name.clone(),
            None => "no_username".to_string(),
        }
        None => "no_username".to_string(),
    }
}