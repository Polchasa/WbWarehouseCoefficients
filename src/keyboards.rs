use crate::database::{
    count_user_numbers, count_warehouses, get_user_browser_profiles_page, get_warehouses_page,
};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, UserId};

// Функция для создания главного меню (клавиатуры)
pub fn main_menu() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "📍Коэффиценты складов",
            "warehouses_list_callback",
        )],
    ])
}

pub fn to_main_menu_button() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
        "🏠Главное меню",
        "main_menu",
    )]])
}

pub async fn create_warehouse_keyboard(page: i32, page_size: i32) -> InlineKeyboardMarkup {
    let warehouses = get_warehouses_page(page, page_size).await.unwrap();
    let total_warehouses = count_warehouses().await.unwrap();
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];

    // Создаем кнопки для каждого склада
    for w in warehouses {
        buttons.push(vec![InlineKeyboardButton::callback(
            w.name,
            format!("whid:{}", w.id.to_string()),
        )]);
    }

    // Добавляем кнопки перелистывания
    let mut nav_buttons = vec![];
    if page > 0 {
        nav_buttons.push(InlineKeyboardButton::callback(
            "⬅️ Назад",
            format!("w_page:{}", page - 1),
        ));
    }
    if (page + 1) * page_size < total_warehouses {
        nav_buttons.push(InlineKeyboardButton::callback(
            "Вперед ➡️",
            format!("w_page:{}", page + 1),
        ));
    }

    // Добавляем навигационные кнопки в конец клавиатуры
    if !nav_buttons.is_empty() {
        buttons.push(nav_buttons);
    }
    buttons.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "main_menu",
    )]);

    InlineKeyboardMarkup::new(buttons)
}

pub fn create_box_types_keyboard(
    box_types: Vec<String>,
    warehouse_id: i32,
) -> InlineKeyboardMarkup {
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];
    for t in box_types {
        buttons.push(vec![InlineKeyboardButton::callback(
            t.clone(),
            format!("boxtype:{} whid:{}", t, warehouse_id),
        )]);
    }
    buttons.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "main_menu",
    )]);

    InlineKeyboardMarkup::new(buttons)
}

pub fn create_coefficents_keyboard(btype: i32, _page: i32) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(
            "📦Выбрать другой тип поставки",
            format!("another_box_type_callback:{}", btype),
        )],
        vec![InlineKeyboardButton::callback(
            "📍Выбрать другой склад",
            format!("another_warehouse_callback"),
        )],
        vec![InlineKeyboardButton::callback(
            "🏠Главное меню",
            "main_menu",
        )],
    ])
}

pub async fn create_user_profiles_keyboard(
    id: UserId,
    page: i32,
    page_size: i32,
) -> InlineKeyboardMarkup {
    let phones = get_user_browser_profiles_page(id, page, page_size)
        .await
        .unwrap();
    let total_warehouses = count_user_numbers(id).await.unwrap();
    let mut buttons: Vec<Vec<InlineKeyboardButton>> = vec![];

    // Создаем кнопки для каждого склада
    for p in phones {
        buttons.push(vec![InlineKeyboardButton::callback(
            format!("+7{}", p),
            format!("phone:{}", p),
        )]);
    }

    // Добавляем кнопки перелистывания
    let mut nav_buttons = vec![];
    if page > 0 {
        nav_buttons.push(InlineKeyboardButton::callback(
            "⬅️ Назад",
            format!("p_page:{}", page - 1),
        ));
    }
    if (page + 1) * page_size < total_warehouses {
        nav_buttons.push(InlineKeyboardButton::callback(
            "Вперед ➡️",
            format!("p_page:{}", page + 1),
        ));
    }

    // Добавляем навигационные кнопки в конец клавиатуры
    if !nav_buttons.is_empty() {
        buttons.push(nav_buttons);
    }
    buttons.push(vec![InlineKeyboardButton::callback(
        "🏠 Главное меню",
        "main_menu",
    )]);

    InlineKeyboardMarkup::new(buttons)
}
