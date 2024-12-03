use std::error::Error;
use base64::decode as base64_decode;
use chrono::{Duration, TimeZone, Utc};
use serde_json::Value;
use std::str;

#[derive(Debug)]
pub enum _Mask {
    Content = 1,            //Контент
    Analytics = 2,          //Аналитика
    PricesDiscounts = 3,   //Цены и скидки
    Marketplace = 4,        //Маркетплейс
    Statistics = 5,         //Статистика
    Promotion = 6,          //Продвижение
    QuestionsFeedback = 7, //Вопросы и отзывы
    Recommendations = 8,    //Рекомендации
    ChatWithBuyers = 9,   //Чат с покупателями
    Supplies = 10,          //Поставки
    CustomerReturns = 11,  //Возвраты покупателям
    Documents = 12,         //Документы
    ReadOnly = 30,         //Токен только на чтение
}

//Получение информации из токена
async fn decode_payload_from_token(token: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let token_str = token.as_str();
    let parts: Vec<&str> = token_str.split('.').collect();
    if parts.len() != 3 {
        return Err("Некорректный токен: токен содержит менее трех частей".into());
    }

    let payload = parts[1];
    let decoded_payload = base64_decode(payload)?;

    let payload_str = String::from_utf8(decoded_payload)
        .map_err(|_| "Ошибка преобразования в строку")?;

    Ok(payload_str)
}

//Получение поля exp из токена
async fn extract_exp_from_token(token: String) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let json: Value = serde_json::from_str(&decode_payload_from_token(token).await?)?;
    let exp = json["exp"].as_i64().ok_or("Поле 'exp' не найдено или имеет неверный тип")?;
    Ok(exp)
}

//Проверка просрочен ли токен
pub async fn is_token_expired(token: String) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let token_exp_time_unix = extract_exp_from_token(token).await?;
    let current_time_unix = Utc::now().timestamp();
    Ok(token_exp_time_unix > current_time_unix)
}

//Получение строки с тем когда выходит срок токена
pub async fn get_lifetime_str(token: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let token_exp_time_unix = extract_exp_from_token(token).await?;
    let token_exp_time = match Utc.timestamp_opt(token_exp_time_unix, 0) {
        chrono::LocalResult::Single(t) => t,
        _ => return Err("Некорректное значение времени".into()),
    };

    let moscow_time = token_exp_time + Duration::hours(3);

    let token_exp_time_string = moscow_time.format("%d.%m.%Y %H:%M:%S").to_string();

    Ok(token_exp_time_string)
}

//Получение поля s из токена (это поле несет в себе свойства, к чему относится токен, поставки и тд)
async fn _extract_s_from_token(token: String) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let json: Value = serde_json::from_str(&decode_payload_from_token(token).await?)?;
    let s = json["s"].as_i64().ok_or("Поле 'exp' не найдено или имеет неверный тип")?;
    Ok(s)
}

pub async fn _is_token_property_set(token: String, bit: _Mask) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let s_bit_mask = _extract_s_from_token(token).await?;
    Ok((s_bit_mask & bit as i64) != 0) 
}