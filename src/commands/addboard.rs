use crate::db;
use serenity::all::ReactionType;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

// find all occurances of the pattern <:text:emoji_id> or unicode emoji
fn parse_reactions(reactions: String) -> Vec<ReactionType> {
    let mut parsed_reactions = Vec::new();

    // really bad implementation
    for reaction in reactions.split_whitespace() {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }
    let split_reactions = reactions.split(">").collect::<Vec<&str>>();
    let len = split_reactions.len();
    let reactions = split_reactions
        .into_iter()
        .take(len - 1)
        .map(|r| r.trim().to_owned() + ">")
        .collect::<Vec<String>>();
    for reaction in reactions {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }

    // filter out duplicates by converting each value back to string
    parsed_reactions.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    parsed_reactions.dedup_by(|a, b| a.to_string() == b.to_string());
    parsed_reactions
}

pub fn run(guild_id: String, options: &[ResolvedOption]) -> String {
    let mut min_reactions: i64 = 1;
    let mut dest_channel: String = String::new();
    let mut name: String = String::new();
    let mut reactions: String = String::new();

    for option in options.iter() {
        match option.name {
            "min-reactions" => match option.value {
                ResolvedValue::Integer(value) => min_reactions = value,
                _ => return "Invalid value for min-reactions".to_string(),
            },
            "dest-channel" => match option.value {
                ResolvedValue::Channel(value) => dest_channel = value.id.to_string(),
                _ => return "Invalid value for dest-channel".to_string(),
            },
            "name" => match option.value {
                ResolvedValue::String(value) => name = value.to_string(),
                _ => return "Invalid value for name".to_string(),
            },
            "reactions" => match option.value {
                ResolvedValue::String(value) => reactions = value.to_string(),
                _ => return "Invalid value for reactions".to_string(),
            },
            _ => return "Invalid option".to_string(),
        }
    }

    let parsed_reactions = parse_reactions(reactions);
    if parsed_reactions.is_empty() {
        return "No reactions provided".to_string();
    }

    if let Err(err) = db::add_board(
        guild_id,
        name,
        parsed_reactions,
        Some(min_reactions),
        dest_channel,
    ) {
        return format!("Failed to create board: {}", err);
    }

    "Board created successfully".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("addboard")
        .description("Create a new board")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "name", "Name of board")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Channel,
                "dest-channel",
                "Channel where the board will post messages",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "reactions",
                "Reactions to use for the board",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "min-reactions",
                "Minimum number of reactions",
            )
            .min_int_value(1)
            .max_int_value(50)
            .required(false),
        )
}
