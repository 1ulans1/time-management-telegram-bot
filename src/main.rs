use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me,
    },
    utils::command::BotCommands,
};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ChooseDate,
    ChooseStartTime {
        date: String,
    },
    ChooseDuration {
        date: String,
        start_time: String,
    },
    AddTaskName {
        date: String,
        start_time: String,
        duration: String,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting time management bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ChooseDate].endpoint(choose_date))
            .branch(dptree::case![State::ChooseStartTime { date }].endpoint(choose_start_time))
            .branch(
                dptree::case![State::ChooseDuration { date, start_time }].endpoint(choose_duration),
            )
            .branch(
                dptree::case![State::AddTaskName { date, start_time, duration }]
                    .endpoint(add_task_name),
            ),
    )
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Enter the date for which you want to create a task (YYYY-MM-DD):")
        .await?;
    dialogue.update(State::ChooseDate).await?;
    Ok(())
}

async fn choose_date(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(date) => {
            bot.send_message(msg.chat.id, "Choose the start time (HH:mm):").await?;
            dialogue.update(State::ChooseStartTime { date: date.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me the date in the correct format.").await?;
        }
    }
    Ok(())
}

async fn choose_start_time(
    bot: Bot,
    dialogue: MyDialogue,
    date: String,
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(start_time) => {
            bot.send_message(
                msg.chat.id,
                "Choose the duration (10 minutes, 30 minutes, 1 hour, 2 hours...):",
            )
                .await?;
            dialogue.update(State::ChooseDuration { date, start_time: start_time.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me the start time in the correct format.").await?;
        }
    }
    Ok(())
}

async fn choose_duration(
    bot: Bot,
    dialogue: MyDialogue,
    (date, start_time): (String, String),
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(duration) => {
            bot.send_message(msg.chat.id, "Enter the task name:").await?;
            dialogue.update(State::AddTaskName { date
                , start_time, duration: duration.into() }).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me the duration in the correct format.").await?;
        }
    }
    Ok(())
}

async fn add_task_name(
    bot: Bot,
    dialogue: MyDialogue,
    (date, start_time, duration): (String, String, String),
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(task_name) => {
            let task_info = format!(
                "Task added:\nDate: {date}\nStart time: {start_time}\nDuration: {duration}\nTask name: {task_name}",
                date = date,
                start_time = start_time,
                duration = duration,
                task_name = task_name
            );
            bot.send_message(msg.chat.id, task_info).await?;
            dialogue.exit().await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me the task name in plain text.").await?;
        }
    }
    Ok(())
}
