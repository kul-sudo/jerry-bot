use std::time::Duration;

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};
use tokio::time::sleep;

pub fn scheduled_messages_run(command: &CommandInteraction, ctx: &Context) -> String {
    let options = command.data.options();
    if let Some(ResolvedOption {
        value: ResolvedValue::String(message),
        ..
    }) = options.first().cloned()
    {
        if let Some(ResolvedOption {
            value: ResolvedValue::Number(sleep_duration),
            ..
        }) = options.get(1).cloned()
        {
            let ctx_http = ctx.http.clone();
            let channel_id = command.channel_id.clone();

            tokio::spawn(async move {
                sleep(Duration::from_secs(sleep_duration as u64)).await;
                channel_id.say(ctx_http, message.to_string()).await.unwrap()
            });
            "fff".to_string()
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    }
}

pub fn scheduled_messages_register() -> CreateCommand {
    CreateCommand::new("schedule")
        .description("schedule a message")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "message_text",
                "the text of the scheduled message",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Number, "send in", "in seconds")
                .required(true),
        )
}
