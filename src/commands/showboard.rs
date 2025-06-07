use crate::db;
use serenity::all::InteractionContext;
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::prelude::*;

pub fn run(guild_id: String, options: &[ResolvedOption]) -> String {
    let mut board_name: String = String::new();

    for option in options.iter() {
        if option.name == "name" {
            match option.value {
                ResolvedValue::String(val) => board_name = val.to_string(),
                _ => return "Invalid value for name".to_string(),
            }
        } else {
            return "Invalid option".to_string();
        }
    }

    let mut output = "## Boards\n".to_string();
    if board_name.is_empty() {
        // show all boards
        let boards = db::get_guild_boards(guild_id).unwrap();
        for board in boards {
            output.push_str(&format!(
                "- {}    **|**    {}    **|**    **{}**    **|**    <#{}>\n",
                board.name, board.reactions, board.min_reactions, board.dest_channel
            ));
        }
        output
    } else {
        // show specific board
        let board = db::get_board(guild_id, board_name).unwrap();
        output.push_str(&format!(
            "- {}    **|**    {}    **|**    **{}**    **|**    <#{}>\n",
            board.name, board.reactions, board.min_reactions, board.dest_channel
        ));
        output
    }
}
pub fn register() -> CreateCommand {
    CreateCommand::new("showboard")
        .description("Displays boards")
        .add_context(InteractionContext::Guild)
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "name", "Name of board")
                .required(false),
        )
}
