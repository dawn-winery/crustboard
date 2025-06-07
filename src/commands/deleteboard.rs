use crate::db;
use crate::{Context, Error};

#[poise::command(slash_command, guild_only, owners_only)]
pub async fn deleteboard(
    ctx: Context<'_>,
    #[description = "Name of the board to delete"] name: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    match db::delete_board(guild_id.to_string(), name.clone()) {
        Ok(()) => {
            ctx.say(format!("Board '{}' deleted successfully!", name))
                .await?;
        }
        Err(err) => {
            ctx.say(format!("Failed to delete board: {}", err)).await?;
        }
    }

    Ok(())
}
