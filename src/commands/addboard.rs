use crate::db;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only, owners_only)]
pub async fn addboard(
    ctx: Context<'_>,
    #[description = "Name of the board"] name: String,
    #[description = "Channel where the board will post messages"]
    dest_channel: serenity::GuildChannel,
    #[description = "Reactions to use for the board (space-separated)"] reactions: String,
    #[description = "Minimum number of reactions"]
    #[min = 1]
    #[max = 50]
    min_reactions: Option<i64>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    let parsed_reactions = crate::commands::parse_reactions(reactions);
    if parsed_reactions.is_empty() {
        ctx.say("No valid reactions provided. Please provide valid Unicode emojis or custom emojis in the format <:name:id>").await?;
        return Ok(());
    }

    match db::add_board(
        guild_id.to_string(),
        name.clone(),
        parsed_reactions,
        min_reactions,
        dest_channel.id.to_string(),
    ) {
        Ok(()) => {
            ctx.say(format!(
                "Board '{}' created successfully! Messages with {} or more reactions will be posted to <#{}>",
                name,
                min_reactions.unwrap_or(1),
                dest_channel.id.to_string()
            )).await?;
        }
        Err(err) => {
            ctx.say(format!("Failed to create board: {}", err)).await?;
        }
    }

    Ok(())
}
