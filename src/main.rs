#![feature(duration_constructors)]
#![feature(allocator_api)]

mod constants;

use std::alloc::Global;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use ::tokio::time::interval;
use constants::THEROCK_EMOJI;
use rand::{thread_rng, Rng};
use serenity::all::{
    ChannelId, Command, CommandOptionType, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, Message,
    ReactionType, Ready, ResolvedOption, ResolvedValue, User,
};
use serenity::async_trait;
use serenity::model::timestamp::Timestamp;
use serenity::prelude::*;

struct Handler;

struct OneWordStory;
struct TemporaryMessages;

impl TypeMapKey for OneWordStory {
    type Value = Arc<RwLock<Vec<(String, User)>>>;
}

impl TypeMapKey for TemporaryMessages {
    type Value = Arc<RwLock<Vec<Message>>>;
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
            format!("{} has been warned for **{}**", user, reason)
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

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let content = match command.data.name.as_str() {
                "warn" => Some(warn_run(&command.data.options())),
                "therock" => Some(therock_run(&command.data.options())),
                _ => Some("unimplemented".to_string()),
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

        let commands = vec![warn_register(), therock_register()];
        // //
        // for command in &commands {
        //     Command::create_global_command(&ctx.http, command.clone())
        //         .await
        //         .unwrap();
        // }
        //
        Command::set_global_commands(&ctx.http, vec![])
            .await
            .unwrap();
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

                let temporary_messages_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<TemporaryMessages>().unwrap().clone()
                };

                let mut temporary_messages = temporary_messages_lock.write().await;

                temporary_messages.push(warning_msg);
            }
        });

        tokio::spawn(async move {
            let temporary_messages_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<TemporaryMessages>().unwrap().clone()
            };

            let mut temporary_messages = temporary_messages_lock.write().await;

            let mut new_temporary_messages = Vec::with_capacity(temporary_messages.len());
            for message in temporary_messages.iter() {
                println!("{:?}", Timestamp::now().timestamp());
                println!("{:?}", message.timestamp.timestamp());

                if Timestamp::now().timestamp() >= message.timestamp.timestamp() {
                    message.delete(&ctx.http).await.unwrap();
                    println!("fff");
                } else {
                    new_temporary_messages.push(message.clone())
                };
            }

            temporary_messages.clear();
            for message in new_temporary_messages {
                temporary_messages.push(message)
            }
        });
    }
    async fn message(&self, ctx: Context, msg: Message) {
        let words_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<OneWordStory>().unwrap().clone()
        };

        let mut words = words_lock.write().await;

        // let words = data.get_mut::<OneWordStory>().unwrap();
        // let temporary_messages = data.get_mut::<TemporaryMessages>().unwrap();

        let msg_is_mine = msg.author == **ctx.cache.current_user();

        if msg.channel_id == 1239201355722264576 && !msg_is_mine {
            if let Some((_, last_story_contributor)) = words.last() {
                if *last_story_contributor == msg.author {
                    msg.react(&ctx.http, ReactionType::Unicode("❌".to_string()))
                        .await
                        .unwrap();

                    let warning_msg = msg
                        .channel_id
                        .say(
                            &ctx.http,
                            format!("{} you can't type 2 words in a row", msg.author),
                        )
                        .await
                        .unwrap();

                    let temporary_messages_lock = {
                        let data_read = ctx.data.read().await;
                        data_read.get::<TemporaryMessages>().unwrap().clone()
                    };

                    let mut temporary_messages = temporary_messages_lock.write().await;
                    temporary_messages.push(warning_msg);

                    return;
                }
            }

            if msg.content.split_whitespace().collect::<Vec<_>>().len() != 1 {
                msg.react(&ctx.http, ReactionType::Unicode("❌".to_string()))
                    .await
                    .unwrap();

                let warning_msg = msg
                    .channel_id
                    .say(&ctx.http, format!("{} only 1 word is allowed", msg.author))
                    .await
                    .unwrap();

                let temporary_messages_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<TemporaryMessages>().unwrap().clone()
                };

                let mut temporary_messages = temporary_messages_lock.write().await;
                temporary_messages.push(warning_msg);
                return;
            }

            if msg.content.ends_with('.') {
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

            words.push((msg.content.clone(), msg.author.clone()));

            msg.react(&ctx.http, ReactionType::Unicode("✅".to_string()))
                .await
                .unwrap();

            return;
        }

        if msg.content.contains(":gigachad:")
            && !msg_is_mine
            && thread_rng().gen_range(0.0..1.0) <= 0.08
        {
            msg.channel_id.say(&ctx.http, ":gigachad:").await.unwrap();
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
        data.insert::<TemporaryMessages>(Arc::new(RwLock::new(Vec::new())));
    }

    client.start().await.unwrap();
}