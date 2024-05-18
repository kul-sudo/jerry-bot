use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

use crate::{CurrentSentence, OneWordStory};

pub async fn story_run(options: &[ResolvedOption<'_>], ctx: &Context) -> String {
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
                let mut words = words_lock.read().await.clone();

                let current_sentence_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<CurrentSentence>().unwrap().clone()
                };

                let current_sentence = current_sentence_lock.read().await;

                for (word, _) in current_sentence.iter() {
                    words.push(word.clone());
                }

                if words.is_empty() {
                    "The story is empty.".to_string()
                } else {
                    words
                        .iter()
                        .map(|word| word.as_str())
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
                    .map(|word| word.as_str())
                    .collect::<Vec<_>>()
                    .join(" ");

                words.clear();

                let current_sentence_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<CurrentSentence>().unwrap().clone()
                };

                let mut current_sentence = current_sentence_lock.write().await;
                current_sentence.clear();

                story
            }
            _ => "Unrecognized command.".to_string(),
        }
    } else {
        "No command".to_string()
    }
}

pub fn story_register() -> CreateCommand {
    CreateCommand::new("story").description("story").add_option(
        CreateCommandOption::new(CommandOptionType::String, "command", "see").required(true),
    )
}
