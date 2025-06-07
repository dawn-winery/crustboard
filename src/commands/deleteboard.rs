use crate::db;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub fn run(guild_id: String, options: &[ResolvedOption]) -> String {
    if let Some(option) = options.first() {
        if option.name == "name" {
            match option.value {
                ResolvedValue::String(value) => match db::delete_board(guild_id, value.to_string())
                {
                    Ok(_) => return "Board deleted successfully".to_string(),
                    Err(err) => return format!("Error deleting board: {}", err),
                },
                _ => return "Invalid argument".to_string(),
            }
        }
    }

    "No value provided".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("deleteboard")
        .description("Deletes a board")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "name",
                "The name of the board to delete",
            )
            .required(true),
        )
}
