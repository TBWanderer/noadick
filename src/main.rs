use dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::pin::Pin;
use teloxide::{prelude::*, utils::command::BotCommands};

#[derive(Serialize, Deserialize)]
struct DickOwner {
    name: String,
    size: i16,
    last: u128,
}
type DickOwners = HashMap<i64, DickOwner>;

fn load(chat_id: i64) -> anyhow::Result<DickOwners> {
    let dir = std::env::var("STORAGE_PATH").unwrap_or("./".into());
    if !fs::exists(&dir)? {
        fs::create_dir(&dir)?;
    }
    let path = dir + &format!("./{}.json", chat_id);
    if fs::exists(&path)? {
        let data = fs::read(&path)?;
        let owners: DickOwners = serde_json::from_str(&String::from_utf8(data)?)?;
        return Ok(owners);
    } else {
        save(chat_id, DickOwners::new())?;
        return Ok(DickOwners::new());
    }
}

fn save(chat_id: i64, owners: DickOwners) -> anyhow::Result<()> {
    let dir = std::env::var("STORAGE_PATH").unwrap_or("./".into());
    if !fs::exists(&dir)? {
        fs::create_dir(&dir)?;
    }
    let path = dir + &format!("./{}.json", chat_id);
    fs::write(path, serde_json::to_string(&owners)?)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Каманды: ")]
enum Command {
    #[command(description = "вывести этот текст")]
    Help,
    #[command(description = "испытать удачу")]
    Dick,
    #[command(description = "вывести топ список")]
    Top,
}

fn build_send(
    bot: &Bot,
    msg: &Message,
) -> impl Fn(String) -> Pin<Box<dyn Future<Output = ResponseResult<()>> + Send>> {
    let bot = bot.clone();
    let chat_id = msg.chat.id;
    let thread_id = msg.thread_id;

    move |text: String| {
        let bot = bot.clone();
        Box::pin(async move {
            let mut request = bot
                .send_message(chat_id, text)
                .parse_mode(teloxide::types::ParseMode::Html);
            if let Some(thread_id) = thread_id {
                request = request.message_thread_id(thread_id);
            }
            request.await.map(|_| ())
        })
    }
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    use std::time::SystemTime;
    let send = build_send(&bot, &msg);
    match cmd {
        Command::Help => send(Command::descriptions().to_string()).await?,
        Command::Dick => {
            let mut storage = load(msg.chat.id.0).expect("Load error");
            let user = msg.from.clone().unwrap();
            let user_id = user.id.0 as i64;
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time error")
                .as_secs() as u128;

            if let Some(owner) = storage.get(&user_id) {
                let time_diff = current_time - owner.last;
                if time_diff < 24 * 60 * 60 {
                    let time_remaining = 24 * 60 * 60 - time_diff;
                    let hours = time_remaining / 3600;
                    let minutes = (time_remaining % 3600) / 60;
                    send(format!("Попробуй через {} ч {} мин", hours, minutes)).await?;
                    return Ok(());
                }
            }

            let num = rand::random_range(-10..=14);

            storage
                .entry(user_id)
                .and_modify(|owner| {
                    owner.size += num;
                    owner.last = current_time;
                })
                .or_insert(DickOwner {
                    name: user.first_name.clone(),
                    size: num,
                    last: current_time,
                });

            let new_size = storage.get(&user_id).unwrap().size;

            let mut players: Vec<_> = storage.iter().collect();
            players.sort_by(|a, b| b.1.size.cmp(&a.1.size));
            let rank = players.iter().position(|(id, _)| **id == user_id).unwrap() + 1;

            let next_attempt_time = current_time + 24 * 60 * 60;
            let time_until_next = next_attempt_time - current_time;
            let hours = time_until_next / 3600;
            let minutes = (time_until_next % 3600) / 60;

            let user_mention = format!(
                "<a href=\"tg://user?id={}\">{}</a>",
                user.id.0, user.first_name
            );

            let change_text = if num >= 0 {
                "вырос"
            } else {
                "уменьшился"
            };

            let response = format!(
                "{}, твой писюн {} на {} см.\nТеперь он равен {} см.\nТы занимаешь {} место в топе.\nСледующая попытка через {} ч {} мин!",
                user_mention,
                change_text,
                num.abs(),
                new_size,
                rank,
                hours,
                minutes
            );

            send(response).await?;

            save(msg.chat.id.0, storage).expect("Save error");
        }
        Command::Top => {
            let storage = load(msg.chat.id.0).expect("Load error");

            if storage.is_empty() {
                send("😥 Пока нет игроков\nПрисоединяйтесь введя /dick".into()).await?;
                return Ok(());
            }

            let mut players: Vec<_> = storage.iter().collect();
            players.sort_by(|a, b| b.1.size.cmp(&a.1.size));

            let top_players = players.iter().take(10);

            let mut response = String::from("🏆 Топ 10:\n\n");
            for (index, (_, owner)) in top_players.enumerate() {
                response.push_str(&format!(
                    "{}. {} - {} см\n",
                    index + 1,
                    owner.name,
                    owner.size
                ));
            }

            send(response).await?;
        }
    };
    Ok(())
}
