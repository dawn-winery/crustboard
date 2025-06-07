use crate::db;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only, owners_only)]
pub async fn editboard(
    ctx: Context<'_>,
    #[description = "Name of the board to edit"] name: String,
    #[description = "New name for the board"] new_name: Option<String>,
    #[description = "New destination channel"] dest_channel: Option<serenity::GuildChannel>,
    #[description = "New reactions (space-separated)"] reactions: Option<String>,
    #[description = "New minimum number of reactions"]
    #[min = 1]
    #[max = 50]
    min_reactions: Option<i64>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    // check if board exists
    if db::get_board(guild_id.to_string(), name.clone()).is_err() {
        ctx.say(format!("Board '{}' not found!", name)).await?;
        return Ok(());
    }

    // parse reactions if applicable
    let parsed_reactions = if let Some(reactions_str) = reactions.clone() {
        let parsed = crate::commands::parse_reactions(reactions_str);
        if parsed.is_empty() {
            ctx.say("No valid reactions provided. Please provide valid Unicode emojis or custom emojis in the format <:name:id>").await?;
            return Ok(());
        }
        Some(parsed)
    } else {
        None
    };

    match db::edit_board(
        guild_id.to_string(),
        name.clone(),
        new_name.clone(),
        parsed_reactions,
        min_reactions,
        dest_channel.as_ref().map(|c| c.id.to_string()),
    ) {
        Ok(()) => {
            let mut changes = Vec::new();
            if let Some(new_name) = new_name {
                changes.push(format!("name → {}", new_name));
            }
            if let Some(channel) = dest_channel {
                changes.push(format!("destination → <#{}>", channel.id.to_string()));
            }
            if reactions.is_some() {
                changes.push("reactions updated".to_string());
            }
            if let Some(min) = min_reactions {
                changes.push(format!("min reactions → {}", min));
            }

            let changes_str = if changes.is_empty() {
                "No changes made".to_string()
            } else {
                changes.join(", ")
            };

            ctx.say(format!(
                "Board '{}' updated successfully! Changes: {}",
                name, changes_str
            ))
            .await?;
        }
        Err(err) => {
            ctx.say(format!("Failed to edit board: {}", err)).await?;
        }
    }

    Ok(())
}
