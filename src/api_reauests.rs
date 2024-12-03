use reqwest::{Client, header::{HeaderMap, HeaderValue}};
use crate::database::{insert_warehouses, add_or_update_warehouse_coefficents};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Deserialize)]
struct PingResponse {
    #[serde(rename = "TS")]   // Переименовываем для соответствия API
    #[allow(dead_code)] // Подавляем варнинг, поле нужно только для десериализации, но пока не используется
    ts: String,
    #[serde(rename = "Status")]   // Переименовываем для соответствия API
    status: String,
}

#[derive(Deserialize)]
pub struct Warehouse {
    #[serde(rename = "ID")] 
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CoefficientResponse {
    pub date: String,
    pub coefficient: i32,
    #[serde(rename = "warehouseID")]
    pub warehouse_id: u32,
    #[serde(rename = "warehouseName")]
    pub warehouse_name: String,
    #[serde(rename = "boxTypeName")]
    pub box_type_name: String,
    #[serde(rename = "boxTypeID")]
    pub box_type_id: Option<u32>, // boxTypeID может не быть
}

pub async fn check_token(api_key: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let url = "https://common-api.wildberries.ru/ping";

    // Создаем заголовок Authorization с токеном
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(api_key)?);

    let client = reqwest::Client::new();
    let response = client.get(url).headers(headers).send().await?;

    // Проверяем, успешен ли статус ответа
    if response.status().is_success() {
        // Десериализуем тело ответа в структуру PingResponse
        let body: PingResponse = response.json().await?;
        // Проверяем, если "Status" == "OK", возвращаем true
        return Ok(body.status == "OK");
    }

    // Если статус ответа неуспешный, возвращаем false
    Ok(false)
}

pub async fn fetch_warehouses(api_key: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = "https://supplies-api.wildberries.ru/api/v1/warehouses";

    // Создаем заголовок Authorization с токеном
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(api_key)?);

    // Создаем HTTP клиент
    let client = Client::new();
    
    // Отправляем запрос и получаем ответ
    let response = client.get(url)
        .headers(headers)
        .send()
        .await?;

    // Проверяем успешность ответа
    if response.status().is_success() {
        // Десериализуем тело ответа в список складов
        let mut warehouses: Vec<Warehouse> = response.json().await?;
        warehouses.sort_by(|a, b| a.name.cmp(&b.name));
        insert_warehouses(warehouses).await?;
        Ok(())
    } else {
        // Логируем ошибку если запрос неуспешный
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        eprintln!("Ошибка: статус {} и сообщение {}", status, text);
        Err("Не удалось получить список складов".into())
    }
}

pub async fn fetch_and_store_coefficients(api_key: &str, warehouse_ids: Option<Vec<u32>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = "https://supplies-api.wildberries.ru/api/v1/acceptance/coefficients";

    // Создаем заголовок Authorization с токеном
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(api_key)?);

    let client = reqwest::Client::new();
    let mut request = client.get(url).headers(headers);

    // Добавляем параметр warehouseIDs, если он есть
    if let Some(ids) = warehouse_ids {
        let ids_string = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        request = request.query(&[("warehouseIDs", ids_string)]);
    }

    let response = request.send().await?;

    if response.status().is_success() {
        // Десериализуем ответ как опцию, чтобы обработать null
        let coefficients: Option<Vec<CoefficientResponse>> = response.json().await?;
        
        // Проверяем, не вернул ли API null
        if let Some(coefficients) = coefficients {
            // Если данные есть, сохраняем их
            add_or_update_warehouse_coefficents(coefficients).await?;
        } else {
            // Если API вернул null
            eprintln!("В функции fetch_and_store_coefficients API вернул null");
            return Err("WB не дал данных по складу".into());
        }
    }

    Ok(())
}