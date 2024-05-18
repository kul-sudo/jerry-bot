use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

pub fn warn_run(options: &[ResolvedOption]) -> String {
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

pub fn warn_register() -> CreateCommand {
    CreateCommand::new("warn")
        .description("warn a user")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "user", "the user").required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "reason", "why?").required(true),
        )
}
