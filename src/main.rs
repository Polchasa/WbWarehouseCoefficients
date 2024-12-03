// src/main.rs
mod api_reauests;
mod bot_commands;
mod bot_callbacks;
mod database;
mod token_decoder;
mod keyboards;
mod commands_handlers;
mod callback_handlers;

use bot_commands::answer;
use bot_callbacks::callback_handler;
use commands_handlers::bot_started_msg;
use log::info;
use teloxide::prelude::*;
use tokio::runtime::Builder;
use tokio::time::{self, Duration};
use tokio::task;
use std::error::Error;
use database::delete_expired_records;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    info!("Запуск бота для работы с WB");

    database::initialize_db().map_err(|e| {
        eprintln!("Ошибка при иницилизации базы данных: {}", e);
        e
    })?;

    let bot = Bot::from_env();

    let mut delete_interval = time::interval(Duration::from_secs(1*60*60));

    // Создаем задачу для автоудаления
    task::spawn(async move {
        loop {
            delete_interval.tick().await;

            if let Err(e) = delete_expired_records().await {
                eprintln!("Ошибка при удалении старых записей: {:?}", e);
            }
        }
    });

    bot_started_msg(bot.clone()).await?;

    let runtime = Builder::new_multi_thread()
        .worker_threads(4) // Количество потоков в рантайме
        .thread_stack_size(8 * 1024 * 1024) // Размер стека в байтах (например, 3 MB)
        .enable_all()
        .build()?;

    let handler = dptree::entry()
        //.branch(Update::filter_message().endpoint(answer));
        .branch(Update::filter_message().endpoint(|bot, msg| async move { answer(bot, msg).await }))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    runtime
        .spawn(async {
            Dispatcher::builder(bot, handler)
                .enable_ctrlc_handler()
                .build()
                .dispatch()
                .await;
        })
        .await?;

    Ok(())
}
