use std::error::Error;
use teloxide::{prelude::*, types::CallbackQuery, Bot};

use crate::api_reauests::{fetch_and_store_coefficients, fetch_warehouses};
use crate::database::*;
use crate::keyboards::*;
use crate::token_decoder::*;

pub async fn main_menu_callback(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        bot.send_message(message.chat().id, "Главное меню")
            .reply_markup(main_menu())
            .await?;
        set_user_state(q.from.id, State::Idle).await?;
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции main_menu_callback из callback_handlers.rs".into())
    }
}

pub async fn token_lifetime_callback(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        if get_user_state(q.from.id).await? == State::TokenEntered {
            let token = get_user_token(q.from.id).await?;
            let msg_to_user = format!("Токен действителен до {}", get_lifetime_str(token).await?);
            bot.delete_message(message.chat().id, message.id()).await?;
            bot.send_message(message.chat().id, msg_to_user).await?;
            bot.send_message(message.chat().id, "Главное меню")
                .reply_markup(main_menu())
                .await?;
        } else {
            bot.send_message(message.chat().id, "Токен не был введен")
                .await?;
        }
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции token_lifetime_callback из callback_handlers.rs".into())
    }
}

pub async fn warehouses_list_callback(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        match get_user_token(q.from.id).await {
            Ok(token) => {
                if token.is_empty() {
                    set_user_state(q.from.id, State::AwaitingToken).await?;
                    bot.send_message(q.from.id, "Введите токен\n\nТокен должен быть создан для работы с категорией <b>'Поставки'</b>")
                    .parse_mode(teloxide::types::ParseMode::Html)
                    .reply_markup(to_main_menu_button())
                    .await?;
                    Ok(())
                } else {
                    if is_token_expired(token.clone()).await? {
                        fetch_warehouses(&token).await?;
                        bot.edit_message_text(message.chat().id, message.id(), "Выберите склад")
                            .await?;
                        bot.edit_message_reply_markup(message.chat().id, message.id())
                            .reply_markup(create_warehouse_keyboard(0, 10).await)
                            .await?;
                        Ok(())
                    } else {
                        set_user_state(q.from.id, State::AwaitingToken).await?;
                        let msg_to_user = format!("Срок действия токена истек. Действовал до {}.\n\nВведите новый токен\n\nТокен должен быть создан для работы с категорией <b>'Поставки'</b>", get_lifetime_str(token.clone()).await?);
                        bot.send_message(message.chat().id, msg_to_user)
                            .parse_mode(teloxide::types::ParseMode::Html)
                            .reply_markup(to_main_menu_button())
                            .await?;
                        Ok(())
                    }
                }
            }
            Err(e) => {
                bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
                    .await?;
                bot.send_message(message.chat().id, "Главное меню")
                    .reply_markup(main_menu())
                    .await?;
                return Err(format!("Ошибка при работе функции warehouses_list_callback из callback_handlers.rs: {}", e).into());
            }
        }
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouses_list_callback из callback_handlers.rs".into())
    }
}

pub async fn warehouses_page_callback(
    bot: Bot,
    q: CallbackQuery,
    page: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        bot.edit_message_reply_markup(message.chat().id, message.id())
            .reply_markup(create_warehouse_keyboard(page, 10).await)
            .await?;
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouses_page_callback из callback_handlers.rs".into())
    }
}

pub async fn phone_page_callback(
    bot: Bot,
    q: CallbackQuery,
    page: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        bot.edit_message_reply_markup(message.chat().id, message.id())
            .reply_markup(create_user_profiles_keyboard(q.from.id, page, 10).await)
            .await?;
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouses_page_callback из callback_handlers.rs".into())
    }
}

pub async fn warehouse_choosed_callback(
    bot: Bot,
    q: CallbackQuery,
    warehouse_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(ref message) = q.message {
        let token = get_user_token(q.from.id).await?;
        match fetch_and_store_coefficients(&token, Some(vec![warehouse_id.try_into()?])).await {
            Ok(()) => {
                // Обработка успешного выполнения
                match get_unique_box_types(warehouse_id).await {
                    Ok(box_types) => {
                        bot.edit_message_text(
                            message.chat().id,
                            message.id(),
                            "Выберите тип поставки",
                        )
                        .await?;
                        bot.edit_message_reply_markup(message.chat().id, message.id())
                            .reply_markup(create_box_types_keyboard(box_types, warehouse_id))
                            .await?;
                    }
                    Err(e) => eprintln!("Ошибка при получении данных: {:?}", e),
                }
                Ok(())
            }
            Err(e) => {
                // Обработка ошибки  create_warehouse_keyboard(0, 10)
                bot.edit_message_text(
                    message.chat().id,
                    message.id(),
                    "WB не предоставил информации по данному складу\nВыберите другой склад",
                )
                .reply_markup(create_warehouse_keyboard(0, 10).await)
                .await?;
                eprintln!(
                    "Произошла ошибка в функции fetch_and_store_coefficients: {:?}",
                    e
                );
                Err(e)
            }
        }
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouse_choosed из callback_handlers.rs".into())
    }
}

pub async fn box_type_choosed_callback(
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match parse_callback_boxtype_text(&q.data.unwrap()) {
        Some((boxtype, whid)) => {
            if let Some(message) = q.message {
                let msg_to_user = get_warehouse_data(whid, boxtype.clone()).await?;
                bot.delete_message(message.chat().id, message.id()).await?;
                bot.send_message(message.chat().id, msg_to_user)
                    .reply_markup(create_coefficents_keyboard(whid, whid))
                    .await?;
                Ok(())
            } else {
                bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
                    .await?;
                Err("Ошибка при работе функции box_type_choosed_callback из callback_handlers.rs: не удалось получить message".into())
            }
        }
        None => {
            bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
                .await?;
            Err("Ошибка при работе функции parse_callback_boxtype_text из callback_handlers.rs: не удалось получить текст коллбэка".into())
        }
    }
}

fn parse_callback_boxtype_text(callback_text: &str) -> Option<(String, i32)> {
    let parts: Vec<&str> = callback_text.split_whitespace().collect();

    let mut boxtype = String::new();
    let mut whid: i32 = 0;

    for part in parts {
        if part.starts_with("boxtype:") {
            boxtype = part.trim_start_matches("boxtype:").to_string();
            if boxtype == "QR-поставка" {
                boxtype = "QR-поставка с коробами".to_string();
            }
        } else if part.starts_with("whid:") {
            whid = part[5..].parse().unwrap_or(0);
        }
    }

    if !boxtype.is_empty() {
        Some((boxtype, whid))
    } else {
        None
    }
}

pub async fn another_warehouse_callback(
    bot: Bot,
    q: CallbackQuery,
    msg: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        bot.send_message(message.chat().id, msg)
            .reply_markup(create_warehouse_keyboard(0, 10).await)
            .await?;
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouses_page_callback из callback_handlers.rs".into())
    }
}

pub async fn another_box_type_callback(
    bot: Bot,
    q: CallbackQuery,
    warehouse_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(message) = q.message {
        match get_unique_box_types(warehouse_id).await {
            Ok(box_types) => {
                bot.send_message(message.chat().id, "Выберите тип поставки")
                    .reply_markup(create_box_types_keyboard(box_types, warehouse_id))
                    .await?;
            }
            Err(e) => eprintln!("Ошибка при получении данных: {:?}", e),
        }
        Ok(())
    } else {
        bot.send_message(q.from.id, "Внутренняя ошибка, попробуйте повторить позже")
            .await?;
        Err("Ошибка при работе функции warehouse_choosed из callback_handlers.rs".into())
    }
}
