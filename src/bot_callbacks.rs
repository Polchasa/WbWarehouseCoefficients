use std::error::Error;
use teloxide::prelude::*;
use crate::callback_handlers::*;

pub async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(data) = q.clone().data {
        bot.answer_callback_query(q.clone().id).await?; //Ответ телеге что мы приняли коллбэк с клавиши клавиатуры

        match data.as_str() {
            "main_menu" => {
                main_menu_callback(bot, q).await?;
            }
            "token_lifetime_callback" => {
                token_lifetime_callback(bot, q).await?;
            }
            "warehouses_list_callback" => {
                warehouses_list_callback(bot, q).await?;
            }
            "another_warehouse_callback" => {
                another_warehouse_callback(bot, q, "Выберите другой склад".to_string()).await?;
            }
            data if data.starts_with("another_box_type_callback:") => {
                let warehouse_id = data.trim_start_matches("another_box_type_callback:").trim().parse::<i32>().ok().unwrap();
                another_box_type_callback(bot, q, warehouse_id).await?;
            }
            data if data.starts_with("w_page:") => {
                let page: i32 = data[7..].parse().unwrap_or(0);
                warehouses_page_callback(bot, q, page).await?;
            }
            data if data.starts_with("p_page:") => {
                let page: i32 = data[7..].parse().unwrap_or(0);
                phone_page_callback(bot, q, page).await?;
            }
            data if data.starts_with("whid:") => {
                let warehouse_id: i32 = data[5..].parse().unwrap_or(0);
                warehouse_choosed_callback(bot, q, warehouse_id).await?;
            }
            data if data.starts_with("boxtype:") => {
                box_type_choosed_callback(bot, q).await?;
            }
            _ => {
                bot.send_message(q.from.id, format!("Неизвестный callback: {}", data)).await?;
            }
        }
    }
    Ok(())
}