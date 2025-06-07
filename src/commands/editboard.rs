use crate::db;
use serenity::all::InteractionContext;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::*;

pub fn run(guild_id: String, options: &[ResolvedOption]) -> String {
    let mut name = String::new();
    let mut new_name: Option<String> = None;
    let mut new_reactions: Option<String> = None;
    let mut new_min_reactions: Option<i64> = None;
    let mut new_dest_channel: Option<String> = None;

    for option in options.iter() {
        match option.name {
            "name" => match option.value {
                ResolvedValue::String(value) => name = value.to_string(),
                _ => return "Invalid value for name".to_string(),
            },
            "new-name" => match option.value {
                ResolvedValue::String(value) => new_name = Some(value.to_string()),
                _ => return "Invalid value for new-name".to_string(),
            },
            "min-reactions" => match option.value {
                ResolvedValue::Integer(value) => new_min_reactions = Some(value),
                _ => return "Invalid value for min-reactions".to_string(),
            },
            "dest-channel" => match option.value {
                ResolvedValue::Channel(value) => new_dest_channel = Some(value.id.to_string()),
                _ => return "Invalid value for dest-channel".to_string(),
            },
            "reactions" => match option.value {
                ResolvedValue::String(value) => new_reactions = Some(value.to_string()),
                _ => return "Invalid value for reactions".to_string(),
            },
            _ => return "Invalid option".to_string(),
        }
    }

    let parsed_reactions = {
        if let Some(new_reactions) = new_reactions {
            Some(crate::commands::parse_reactions(new_reactions))
        } else {
            None
        }
    };

    if let Err(err) = db::edit_board(
        guild_id,
        name,
        new_name,
        parsed_reactions,
        new_min_reactions,
        new_dest_channel,
    ) {
        return format!("Failed to edit board: {}", err);
    }

    "Board edited successfully".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("editboard")
        .description("Edit a board")
        .add_context(InteractionContext::Guild)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "The name of the board to edit",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "new-name",
                "The new name of the board",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "reactions",
                "The new reactions of the board",
            )
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "min-reactions",
                "The new minimum number of reactions required to pin",
            )
            .min_int_value(1)
            .max_int_value(50)
            .required(false),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "dest-channel",
                "The new channel of the board",
            )
            .required(false),
        )
}
