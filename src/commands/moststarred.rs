use crate::{
    Context, Error,
    commands::{autocomplete_board_names, create_reply},
    db,
};
use poise::serenity_prelude::*;

#[poise::command(slash_command, guild_only)]
pub async fn moststarred(
    ctx: Context<'_>,
    #[description = "The name of the board to check"]
    #[autocomplete = "autocomplete_board_names"]
    name: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    let message_data = {
        if let Some(ref name) = name {
            db::get_board_messages(guild_id, name)
        } else {
            db::get_guild_messages(guild_id)
        }
    };

    match message_data {
        Ok(data) => {
            if let Some(max) = data
                .iter()
                .max_by(|a, b| a.reaction_count.cmp(&b.reaction_count))
            {
                let board = db::get_board_by_id(&max.board_id).unwrap();

                let message_id = MessageId::new(max.dest_id.parse().unwrap());
                let channel_id = ChannelId::new(board.dest_channel.parse().unwrap());

                match ctx.http().get_message(channel_id, message_id).await {
                    Ok(message) => match create_reply(ctx, message).await {
                        Ok(reply) => {
                            ctx.send(reply).await?;
                        }
                        Err(err) => {
                            ctx.say(format!("Could not create reply: {}", err)).await?;
                        }
                    },
                    Err(err) => {
                        ctx.say(format!("Message not found: {}", err)).await?;
                    }
                }
            } else {
                ctx.say("No maximum found").await?;
            }
        }
        Err(err) => {
            ctx.say(format!("Error fetching message data: {}", err))
                .await?;
        }
    }

    Ok(())
}
