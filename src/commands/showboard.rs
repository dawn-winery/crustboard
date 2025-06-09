use crate::{Context, Error, commands::autocomplete_board_names, db};

#[poise::command(slash_command, guild_only)]
pub async fn showboard(
    ctx: Context<'_>,
    #[description = "Name of a specific board to show (optional)"]
    #[autocomplete = "autocomplete_board_names"]
    name: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    match name {
        Some(board_name) => {
            // specific board
            match db::get_board(guild_id, &board_name) {
                Ok(board) => {
                    let reactions = db::from_csv(board.reactions);
                    let reaction_str = reactions
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<_>>()
                        .join(" ");

                    ctx.say(format!(
                        "**Board: {}**\n**Destination:** <#{}>\n**Reactions:** {}\n**Min Reactions:** {}",
                        board.name,
                        board.dest_channel,
                        reaction_str,
                        board.min_reactions
                    )).await?;
                }
                Err(_) => {
                    ctx.say(format!("Board '{}' not found!", board_name))
                        .await?;
                }
            }
        }
        None => {
            // all boards
            match db::get_guild_boards(guild_id) {
                Ok(boards) => {
                    if boards.is_empty() {
                        ctx.say("No boards found in this server!").await?;
                    } else {
                        let mut response = "**Server Boards:**\n\n".to_string();
                        for board in boards {
                            let reactions = db::from_csv(board.reactions);
                            let reaction_str = reactions
                                .iter()
                                .map(|r| r.to_string())
                                .collect::<Vec<_>>()
                                .join(" ");

                            response.push_str(&format!(
                                "**{}** → <#{}> ({}+ reactions: {})\n",
                                board.name, board.dest_channel, board.min_reactions, reaction_str
                            ));
                        }
                        ctx.say(response).await?;
                    }
                }
                Err(err) => {
                    ctx.say(format!("Failed to retrieve boards: {}", err))
                        .await?;
                }
            }
        }
    }

    Ok(())
}
