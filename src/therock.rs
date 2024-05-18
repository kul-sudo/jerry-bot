use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

use crate::constants::THEROCK_EMOJI;

pub fn therock_run(options: &[ResolvedOption]) -> String {
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

pub fn therock_register() -> CreateCommand {
    CreateCommand::new("therock")
        .description("throw therock at a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "victim", "the victim")
                .required(true),
        )
}
