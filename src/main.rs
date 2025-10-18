use chrono::{Duration, prelude::*};
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
    last: i64,
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
    pretty_env_logger::init();

    match std::env::var("DEBUG") {
        Ok(mode) => match mode.as_str().to_lowercase().as_str() {
            "true" | "1" | "yes" => {
                dotenv::from_filename(".debug.env").ok();
                log::warn!("Using DEBUG .env")
            }
            _ => panic!("Please set correct value to DEBUG ()"),
        },
        Err(_) => {
            dotenv::from_filename(".release.env").ok();
            log::warn!("Using RELEASE .env")
        }
    };
    log::info!("Starting bot...");

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
    let send = build_send(&bot, &msg);
    match cmd {
        Command::Help => send(Command::descriptions().to_string()).await?,
        Command::Dick => {
            let mut storage = load(msg.chat.id.0).expect("Load error");
            let user = msg.from.clone().unwrap();
            let user_id = user.id.0 as i64;
            let now = Local::now();
            let tomorrow_midnight = (now + Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap();
            let duration_to_tomorrow = tomorrow_midnight
                .signed_duration_since(now.naive_local())
                .to_std()
                .unwrap();
            let time_remaining = duration_to_tomorrow.as_secs();
            let hours = time_remaining / 3600;
            let minutes = (time_remaining % 3600) / 60;

            if let Some(owner) = storage.get(&user_id) {
                if (DateTime::from_timestamp(owner.last, 0).unwrap().day() == now.day())
                    && (now.timestamp() - owner.last < 24 * 60 * 60)
                {
                    let mut players: Vec<_> = storage.iter().collect();
                    players.sort_by(|a, b| b.1.size.cmp(&a.1.size));
                    let rank = players.iter().position(|(id, _)| **id == user_id).unwrap() + 1;

                    let user_mention = format!(
                        "<a href=\"tg://user?id={}\">{}</a>",
                        user.id.0, user.first_name
                    );

                    let response = format!(
                        "{}, твой писюн равен {} см.\nТы занимаешь {} место в топе.\nПопробуй через {} ч {} мин",
                        user_mention, owner.size, rank, hours, minutes
                    );
                    send(response).await?;
                    return Ok(());
                }
            }

            let num = {
                fn weighted_range_random(ranges: &[(std::ops::RangeInclusive<i16>, f32)]) -> i16 {
                    use rand::Rng;
                    let mut rng = rand::rng();
                    let total: f32 = ranges.iter().map(|(_, w)| w).sum();
                    let mut roll = rng.random::<f32>() * total;
                    for (range, weight) in ranges {
                        roll -= weight;
                        if roll <= 0.0 {
                            return rng.random_range(range.clone());
                        }
                    }
                    let fallback = &ranges.last().unwrap().0;
                    rng.random_range(fallback.clone())
                }
                let ranges = vec![(-10..=0, 0.3), (1..=7, 0.6), (8..=14, 0.1)];
                weighted_range_random(&ranges)
            };

            storage
                .entry(user_id)
                .and_modify(|owner| {
                    owner.size += num;
                    owner.last = now.timestamp();
                })
                .or_insert(DickOwner {
                    name: user.first_name.clone(),
                    size: num,
                    last: now.timestamp(),
                });

            let new_size = storage.get(&user_id).unwrap().size;
            let mut players: Vec<_> = storage.iter().collect();
            players.sort_by(|a, b| b.1.size.cmp(&a.1.size));
            let rank = players.iter().position(|(id, _)| **id == user_id).unwrap() + 1;
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
                "{}, твой писюн {} на {} см.\nТеперь он равен {} см.\nТы занимаешь {} место в топе.\nСледующая попытка завтра, через {} ч {} мин!",
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
