#![feature(duration_constructors)]
#![feature(allocator_api)]

mod constants;

use std::alloc::Global;
use std::env::{self};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use constants::THEROCK_EMOJI;
use serenity::all::{
    ChannelId, CommandOptionType, CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseMessage, GuildId, Interaction, Message, ReactionType, Ready,
    ResolvedOption, ResolvedValue, User,
};
use serenity::async_trait;
use serenity::prelude::*;
use tokio::time::interval;

struct Handler;

struct OneWordStory;
struct CurrentNumber;

const PUNCTUANTION: [char; 33] = [
    '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', ' ', '-', '.', '/', ':', ';', '<',
    '=', '>', '?', '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
];

macro_rules! make_temp {
    ($msgs:expr, $ctx_http:expr) => {
        tokio::spawn(async move {
            sleep(Duration::from_secs(5));
            for msg in &$msgs {
                msg.delete($ctx_http).await.unwrap();
            }
        });
    };
}

macro_rules! react_positively {
    ($msg:expr, $ctx_http:expr) => {
        $msg.react($ctx_http, ReactionType::Unicode("✅".to_string()))
            .await
            .unwrap();
    };
}

macro_rules! react_negatively {
    ($msg:expr, $ctx_http:expr) => {
        $msg.react($ctx_http, ReactionType::Unicode("❌".to_string()))
            .await
            .unwrap();
    };
}

impl TypeMapKey for OneWordStory {
    type Value = Arc<RwLock<Vec<(String, User)>>>;
}

impl TypeMapKey for CurrentNumber {
    type Value = Arc<RwLock<(isize, Option<User>)>>;
}

fn warn_run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        if let Some(ResolvedOption {
            value: ResolvedValue::String(reason),
            ..
        }) = options.get(1)
        {
            format!("{} has been warned for **{}**.", user, reason)
        } else {
            "No reason".to_string()
        }
    } else {
        "No user".to_string()
    }
}

fn warn_register() -> CreateCommand {
    CreateCommand::new("warn")
        .description("warn a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "the user").required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "reason", "why?").required(true),
        )
}

fn therock_run(options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::User(user, _),
        ..
    }) = options.first()
    {
        format!("{} {}", user, THEROCK_EMOJI)
    } else {
        "No user".to_string()
    }
}

fn therock_register() -> CreateCommand {
    CreateCommand::new("therock")
        .description("throw therock at a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "victim", "the victim")
                .required(true),
        )
}

async fn story_run(options: &[ResolvedOption<'_>], ctx: &Context) -> String {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(command),
        ..
    }) = options.first()
    {
        match *command {
            "see" => {
                let words_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<OneWordStory>().unwrap().clone()
                };
                let words = words_lock.read().await;

                if words.is_empty() {
                    "The story is empty.".to_string()
                } else {
                    words
                        .iter()
                        .map(|(word, _)| word.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            }
            "end" => {
                let words_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<OneWordStory>().unwrap().clone()
                };

                let mut words = words_lock.write().await;

                let story = words
                    .iter()
                    .map(|(word, _)| word.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");
                words.clear();
                story
            }
            _ => "Unrecognized command.".to_string(),
        }
    } else {
        "No command".to_string()
    }
}

fn story_register() -> CreateCommand {
    CreateCommand::new("story").description("story").add_option(
        CreateCommandOption::new(CommandOptionType::String, "command", "see").required(true),
    )
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "warn" => Some(warn_run(&command.data.options())),
                "therock" => Some(therock_run(&command.data.options())),
                "story" => Some(story_run(&command.data.options(), &ctx).await),
                _ => unreachable!(),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        let guild_id = GuildId::new(1238975924725350430);

        let commands = vec![warn_register(), therock_register(), story_register()];
        // //
        // for command in &commands {
        //     Command::create_global_command(&ctx.http, command.clone())
        //         .await
        //         .unwrap();
        // }
        //
        // Command::set_global_commands(&ctx.http, vec![])
        //     .await
        //     .unwrap();
        // guild_id.set_commands(&ctx.http, vec![]).await.unwrap();
        guild_id.set_commands(&ctx.http, commands).await.unwrap();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_hours(3));

            loop {
                interval.tick().await;

                let warning_msg = ChannelId::new(1238975924725350433)
                    .say(
                        &ctx.http,
                        format!("Don't forget to follow the rules! {}", THEROCK_EMOJI),
                    )
                    .await
                    .unwrap();

                let http = ctx.http.clone();

                make_temp!([warning_msg], &http);
            }
        });
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let words_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<OneWordStory>().unwrap().clone()
        };

        let mut words = words_lock.write().await;

        let msg_is_mine = msg.author == **ctx.cache.current_user();

        if msg.channel_id == 1241105245791322203 && !msg_is_mine {
            match msg.content.parse::<isize>() {
                Ok(number) => {
                    let current_number_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<CurrentNumber>().unwrap().clone()
                    };

                    let mut current_number = current_number_lock.write().await;

                    if let Some(user) = &current_number.1 {
                        if *user == msg.author {
                            let err_msg = msg
                                .channel_id
                                .say(&ctx.http, format!("{} You can't count twice.", msg.author))
                                .await
                                .unwrap();

                            react_negatively!(msg, &ctx.http);
                            make_temp!([err_msg, msg], &ctx.http);

                            return;
                        }
                    };

                    if number - current_number.0 == 1 {
                        react_positively!(msg, ctx.http);

                        *current_number = (current_number.0 + 1, Some(msg.author));
                    } else {
                        let err_msg = msg
                            .channel_id
                            .say(
                                &ctx.http,
                                format!("{} You can only increment the number by 1.", msg.author),
                            )
                            .await
                            .unwrap();

                        *current_number = (0, None);

                        react_negatively!(msg, &ctx.http);
                        make_temp!([err_msg, msg], &ctx.http);
                    }
                }
                Err(_) => {
                    let error_msg = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            format!("{} you can only type numbers.", msg.author),
                        )
                        .await
                        .unwrap();

                    react_negatively!(msg, &ctx.http);
                    make_temp!([error_msg], &ctx.http);
                }
            }

            return;
        }

        if msg.channel_id == 1239201355722264576 && !msg_is_mine {
            if let Some((_, last_story_contributor)) = words.last() {
                if *last_story_contributor == msg.author {
                    let warning_msg = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            format!("{} you can't type 2 words in a row.", msg.author),
                        )
                        .await
                        .unwrap();

                    react_negatively!(msg, &ctx.http);
                    make_temp!([warning_msg, msg], &ctx.http);

                    return;
                }
            }

            if msg.content.split_whitespace().collect::<Vec<_>>().len() != 1 {
                let err_msg = msg
                    .channel_id
                    .say(&ctx.http, format!("{} only 1 word is allowed.", msg.author))
                    .await
                    .unwrap();

                react_negatively!(msg, &ctx.http);
                make_temp!([err_msg, msg], &ctx.http);

                return;
            }

            let mut ended = false;

            if PUNCTUANTION
                .iter()
                .any(|char| msg.content.starts_with(*char))
            {
                if words.is_empty() {
                    let error_msg = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            "The story can't be started with a punctuational symbol.",
                        )
                        .await
                        .unwrap();

                    react_negatively!(msg, &ctx.http);
                    make_temp!([error_msg], &ctx.http);

                    return;
                } else if ['.', '?', '!']
                    .iter()
                    .any(|char| msg.content.starts_with(*char) || msg.content.ends_with(*char))
                {
                    ended = true;
                }
            }

            words.push((msg.content.to_string(), msg.author.clone()));

            msg.react(&ctx.http, ReactionType::Unicode("✅".to_string()))
                .await
                .unwrap();

            if ended {
                msg.channel_id
                    .say(
                        &ctx.http,
                        words
                            .iter()
                            .map(|(word, _)| word.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                    )
                    .await
                    .unwrap();

                words.clear();
            }

            return;
        }
    }
}

#[tokio::main]
async fn main() {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(
        env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN expected"),
        intents,
    )
    .event_handler(Handler)
    .await
    .unwrap();

    {
        let mut data = client.data.write().await;
        data.insert::<OneWordStory>(Arc::new(RwLock::new(Vec::new_in(Global))));
        data.insert::<CurrentNumber>(Arc::new(RwLock::new((0, None))))
    }

    client.start().await.unwrap();
}
