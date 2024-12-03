use crate::api_reauests::{Warehouse, CoefficientResponse};
use rusqlite::{params, Connection, Result};
use std::error::Error;
use std::sync::Arc;
use teloxide::types::UserId;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Duration, TimeZone, Utc};

#[derive(PartialEq)]
pub enum State {
    Idle = 0,
    AwaitingToken = 1, //ожидание токена
    TokenEntered = 2,  //токен введен
    AwaitingNumber = 3, //ожидание номера телефона
    AwaitingCaptcha = 4, //ожидание ввода капчи
    AwaitingSMSCode = 5, //ожидаем код из смс
}

impl State {
    fn from_i8(value: i8) -> Option<State> {
        match value {
            1 => Some(State::AwaitingToken),
            2 => Some(State::TokenEntered),
            3 => Some(State::AwaitingNumber),
            4 => Some(State::AwaitingCaptcha),
            5 => Some(State::AwaitingSMSCode),
            _ => Some(State::Idle),
        }
    }
}

pub fn initialize_db() -> Result<()> {
    let conn = Connection::open("bot.db")?;

    let create_tables_query = "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            username TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS user_states (
            id INTEGER PRIMARY KEY,
            state INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS user_tokens (
            chat_id INTEGER PRIMARY KEY,
            token TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS warehouses (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS warehouses_coefficients (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date INTEGER NOT NULL,
            coefficient INTEGER NOT NULL,
            warehouse_id INTEGER NOT NULL,
            warehouse_name TEXT NOT NULL,
            box_type_name TEXT NOT NULL,
            box_type_id INTEGER,
            UNIQUE(date, warehouse_id, box_type_name)
        );
    ";

    conn.execute_batch(create_tables_query)?;

    Ok(())
}

pub async fn get_db_connection() -> Result<Arc<Mutex<Connection>>, Box<dyn Error + Send + Sync>> {
    let conn = Arc::new(Mutex::new(Connection::open("bot.db")?));
    Ok(conn)
}

pub async fn add_user_to_db(
    id: UserId,
    username: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {    
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    conn.execute(
        "INSERT OR IGNORE INTO users (id, username) VALUES (?1, ?2)",
        params![id.0, username],
    )?;
    Ok(())
}

pub async fn _user_exist(id: UserId) -> Result<bool, Box<dyn Error>> {
    let conn = Connection::open("bot.db")?;
    let mut stmt = conn.prepare("SELECT EXISTS(SELECT 1 FROM users WHERE is = ?1)")?;
    let exists: bool = stmt.query_row([id.0], |row| row.get(0))?;
    Ok(exists)
}

pub async fn get_user_ids() -> Result<Vec<i64>, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT id FROM users")?;
    let user_ids_iter = stmt.query_map([], |row| row.get(0))?;

    let mut users_ids: Vec<i64> = Vec::new();
    for user_id_result in user_ids_iter {
        users_ids.push(user_id_result?);
    }

    Ok(users_ids)
}

pub async fn get_id_by_username(username: String) -> Result<i64, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1")?;
    let id: i64 = stmt.query_row([username], |row| row.get(0))?;
    Ok(id)
}

pub async fn set_user_state(
    id: UserId,
    state: State,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    conn.execute(
        "INSERT OR REPLACE INTO user_states (id, state) VALUES (?1, ?2)",
        params![id.0, state as i8],
    )?;
    Ok(())
}

pub async fn _del_user_state(id: UserId) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    conn.execute("DELETE FROM user_states WHERE id = ?1", params![id.0])?;
    Ok(())
}

pub async fn get_user_state(id: UserId) -> Result<State, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;

    let mut stmt = conn.prepare("SELECT state FROM user_states WHERE id = ?1")?;
    let mut rows = stmt.query([id.0])?;

    if let Some(row) = rows.next()? {
        let state: i8 = row.get(0)?;
        match State::from_i8(state) {
            Some(s) => Ok(s),
            None => Err("Неверное состояние пользователя".into()),
        }
    } else {
        Ok(State::Idle)
    }
}

pub async fn set_user_token(
    id: UserId,
    token: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    conn.execute(
        "INSERT OR REPLACE INTO user_tokens (chat_id, token) VALUES (?1, ?2)",
        params![id.0, token],
    )?;
    Ok(())
}

pub async fn get_user_token(id: UserId) -> Result<String, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    let mut stmt = conn.prepare("SELECT token FROM user_tokens WHERE chat_id = ?1")?;
    //let token = stmt.query_row(params![id.0], |row| row.get(0))?;
    let result: Result<String, rusqlite::Error> = stmt.query_row(params![id.0], |row| row.get(0));
    match result {
        Ok(token) => Ok(token),
        Err(e) => {
            if e == rusqlite::Error::QueryReturnedNoRows {
                return Ok(String::new());
            }
            else {
                Err(Box::new(e))
            }
        }
    }
}

pub async fn insert_warehouses(warehouses: Vec<Warehouse>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;

    for warehouse in warehouses {
        conn.execute(
            "INSERT INTO warehouses (id, name) VALUES (?1, ?2)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
            params![warehouse.id, warehouse.name],
        )?;
    }

    Ok(())
}

pub async fn get_warehouses_page(page: i32, page_size: i32) -> Result<Vec<Warehouse>, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    let offset = page * page_size;

    let mut stmt = conn
        .prepare("SELECT id, name FROM warehouses ORDER BY name LIMIT ?1 OFFSET ?2")?;

    let warehouse_iter = stmt
        .query_map([page_size, offset], |row| {
            Ok(Warehouse {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .unwrap();

    let mut warehouses = vec![];
    for warehouse in warehouse_iter {
        if let Ok(w) = warehouse {
            warehouses.push(w);
        }
    }
    Ok(warehouses)
}

pub async fn count_warehouses() -> Result<i32, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM warehouses").unwrap();
    
    let count: i32 = stmt.query_row([], |row| row.get(0))?;

    Ok(count)
}

pub async fn delete_expired_records() -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    // Получаем текущее время в формате Unix timestamp (UTC)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;

    // SQL-запрос для удаления записей с истекшей датой
    let delete_query = "DELETE FROM warehouses_coefficients WHERE date < ?";

    // Выполняем запрос
    conn.execute(delete_query, params![now])?;

    Ok(())
}

pub async fn add_or_update_warehouse_coefficents(coefficients: Vec<CoefficientResponse>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;

    for coefficient in coefficients {
        // SQL-запрос для обновления или вставки записи
        let upsert_query = "
            INSERT INTO warehouses_coefficients (date, coefficient, warehouse_id, warehouse_name, box_type_name, box_type_id)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(date, warehouse_id, box_type_name) DO UPDATE SET
                coefficient = excluded.coefficient;
        ";
        let datetime: DateTime<Utc> = coefficient.date.parse().unwrap();
        let unix_time = datetime.timestamp();
        // Выполняем запрос
        conn.execute(upsert_query, params![unix_time, coefficient.coefficient, coefficient.warehouse_id, coefficient.warehouse_name, coefficient.box_type_name, coefficient.box_type_id])?;
    }

    Ok(())
}

pub async fn get_unique_box_types(warehouse_id: i32) -> Result<Vec<String>> {
    let conn = get_db_connection().await.unwrap();
    let conn = conn.lock().await;

    let mut stmt = conn.prepare("SELECT DISTINCT box_type_name FROM warehouses_coefficients WHERE warehouse_id = ?1 ORDER BY box_type_name COLLATE NOCASE")?;
    let box_type_iter = stmt.query_map([warehouse_id], |row| {
        let box_type_name: String = row.get(0)?;
        Ok(box_type_name)
    })?;

    let mut box_types = Vec::new();
    for box_type in box_type_iter {
        box_types.push(box_type?);
    }

    Ok(box_types)
}

pub async fn get_warehouse_data(wid: i32, btype: String) -> Result<String, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;

    let mut stmt1 = conn.prepare("SELECT name FROM warehouses WHERE id = ?1")?;
    let wname: String = stmt1.query_row(params![wid], |row| row.get(0))?;

    let mut stmt = conn.prepare(
        "SELECT date, coefficient, warehouse_name FROM warehouses_coefficients WHERE warehouse_id = ?1 AND box_type_name = ?2 AND coefficient != -1 ORDER BY coefficient"
    )?;
    
    let mut rows = stmt.query(params![wid, btype])?;
    let mut result = String::new();
    let mut found = false; // Флаг для проверки наличия записей

    result.push_str(&format!("📍Склад: {}\n📦Тип поставки: {}\n\n", wname, btype)); //w_name

    while let Some(row) = rows.next()? {
        found = true;
        let date: i64 = row.get(0)?;
        let coefficient: i32 = row.get(1)?;

        // Преобразование unix времени в формат dd.mm.yyyy hh:mm:ss и перевод в московское время
        let time = Utc.timestamp_opt(date, 0).unwrap();
        let moscow_time = time + Duration::hours(3);
        let moscow_time_string = moscow_time.format("%d.%m.%Y %H:%M:%S").to_string();

        result.push_str(&format!(
            "⌛️Дата: {}\n📈Коэффициент: {}\n\n",
            moscow_time_string, coefficient
        ));
    }
    if !found {
        result.push_str("⛔️Нет доступных поставок");
    }
    Ok(result)
}

pub async fn get_user_browser_profiles_page(
    id: UserId,
    page: i32, 
    page_size: i32
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    let offset = page * page_size;

    let mut stmt = conn
        .prepare("SELECT phone_number FROM chrome_profiles WHERE id = ?1 LIMIT ?2 OFFSET ?3")?;

    let phones_iter = stmt
        .query_map(params![id.0, page_size, offset], |row| {
            let phone: String = row.get(0)?;
            Ok(phone)
        })?;

    let mut phones = vec![];
    for phone in phones_iter {
        if let Ok(w) = phone {
            phones.push(w);
        }
    }
    Ok(phones)
}

pub async fn count_user_numbers(id: UserId) -> Result<i32, Box<dyn Error + Send + Sync>> {
    let conn = get_db_connection().await?;
    let conn = conn.lock().await;
    
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM chrome_profiles WHERE id = ?1").unwrap();
    
    let count: i32 = stmt.query_row([id.0], |row| row.get(0))?;

    Ok(count)
}