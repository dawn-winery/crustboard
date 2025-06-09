use poise::serenity_prelude as serenity;
use poise::{Framework, FrameworkOptions};
use serenity::{
    Client, Context as SerenityContext, GatewayIntents,
    all::{Color, CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage, MessageId, Reaction},
};
use std::env;

mod commands;
pub mod db;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("{} is connected!", data_about_bot.user.name)
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction_add(ctx, add_reaction.clone()).await?
        }

        _ => {}
    }
    Ok(())
}

async fn handle_reaction_add(ctx: &SerenityContext, added: Reaction) -> Result<(), Error> {
    let message = match added.message(&ctx.http).await {
        Ok(message) => message,
        Err(e) => {
            println!("Error getting message: {}", e);
            return Ok(());
        }
    };

    let guild_id = match added.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let users = message
        .channel_id
        .reaction_users(
            ctx.http.clone(),
            message.id,
            added.emoji.clone(),
            None,
            None,
        )
        .await?;

    let count = if users.contains(&message.author) {
        users.len() - 1
    } else {
        users.len()
    };

    let min_reactions = db::find_min_reactions(guild_id.to_string(), added.emoji)?;
    let qualifying_boards = min_reactions
        .iter()
        .filter(|&(_, min, _)| min <= &count)
        .collect::<Vec<_>>();

    let guild_channels = ctx.http.get_channels(guild_id).await?;

    for (board_name, _, dest_channel_id) in qualifying_boards.iter() {
        match db::get_message_dest(guild_id.to_string(), message.id.to_string()) {
            Ok(dest_id) => {
                // update message
                let dest_channel = guild_channels
                    .iter()
                    .find(|channel| channel.id.to_string() == dest_channel_id.to_string())
                    .ok_or("Destination channel not found")?;

                let edit_message = EditMessage::new().content(format!(
                    "{} **| {} Reactions |** <#{}> **({})**",
                    board_name, count, message.channel_id, message.author
                ));

                if let Err(err) = ctx
                    .http
                    .edit_message(
                        dest_channel.id,
                        MessageId::new(dest_id.parse::<u64>()?),
                        &edit_message,
                        Vec::new(),
                    )
                    .await
                {
                    println!("Error editing message: {}", err);
                }

                db::update_message_reaction_count(guild_id, board_name, dest_id, count as i64)?;
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                // create new message
                let dest_message = create_board_message(&message, board_name, count)?;

                // send to destination channel
                let channel = guild_channels
                    .iter()
                    .find(|channel| &channel.id.to_string() == dest_channel_id)
                    .ok_or("Destination channel not found")?;

                let dest_msg = channel.send_message(&ctx.http, dest_message).await?;

                // save to database
                db::add_message(
                    guild_id,
                    board_name,
                    message.author.id,
                    message.id,
                    dest_msg.id,
                    count as i64,
                )?;
            }
            Err(err) => {
                println!("Database error: {}", err);
            }
        }
    }

    Ok(())
}

pub fn create_board_message(
    message: &serenity::Message,
    board_name: impl AsRef<str>,
    count: usize,
) -> Result<CreateMessage, Error> {
    let mut dest_message = CreateMessage::new().content(format!(
        "{} **| {} Reactions |** <#{}> **({})**",
        board_name.as_ref(),
        count,
        message.channel_id,
        message.author
    ));

    // handle reply
    if let Some(referenced_message) = message.referenced_message.clone() {
        dest_message = dest_message.add_embed({
            let mut embed = CreateEmbed::new()
                .author(
                    CreateEmbedAuthor::new(format!(
                        "Replying to {}",
                        referenced_message.author.name
                    ))
                    .url(referenced_message.link())
                    .icon_url(referenced_message.author.avatar_url().unwrap_or_default()),
                )
                .description(referenced_message.content.to_string())
                .timestamp(referenced_message.timestamp);

            if let Some(attachment) = referenced_message.attachments.first() {
                embed = embed.image(attachment.url.to_string());
            }

            embed
        });

        // add additional attachments as embeds
        for attachment in referenced_message.attachments.iter().skip(1) {
            dest_message = dest_message.add_embed(
                CreateEmbed::new()
                    .image(attachment.url.to_string())
                    .color(Color::from_rgb(0x1d, 0xa0, 0xf2)),
            );
        }

        // add referenced message embeds
        for embed in referenced_message.embeds {
            dest_message = dest_message
                .add_embed(CreateEmbed::from(embed).color(Color::from_rgb(0x63, 0x63, 0xff)));
        }
    }

    // add main message embed
    dest_message = dest_message.add_embed({
        let mut embed = CreateEmbed::new()
            .author(
                CreateEmbedAuthor::new(&message.author.name)
                    .url(message.link())
                    .icon_url(message.author.avatar_url().unwrap_or_default()),
            )
            .description(message.content.to_string())
            .color(Color::from_rgb(0xff, 0xe1, 0x9c))
            .timestamp(message.timestamp);

        if let Some(attachment) = message.attachments.first() {
            embed = embed.image(attachment.url.to_string());
        }

        embed
    });

    // add additional attachments
    for attachment in message.attachments.iter().skip(1) {
        dest_message = dest_message.add_embed(
            CreateEmbed::new()
                .image(attachment.url.to_string())
                .color(Color::from_rgb(0x1d, 0xa0, 0xf2)),
        );
    }

    // add message embeds
    for embed in message.embeds.clone() {
        dest_message = dest_message
            .add_embed(CreateEmbed::from(embed).color(Color::from_rgb(0x63, 0x63, 0xff)));
    }

    Ok(dest_message)
}

#[tokio::main]
async fn main() {
    db::create_db().expect("Failed to create database");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![
                commands::addboard(),
                commands::deleteboard(),
                commands::showboard(),
                commands::editboard(),
                commands::leaderboard(),
                commands::moststarred(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Registering commands in {} guilds", ready.guilds.len());
                for guild in &ready.guilds {
                    if let Err(e) = poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild.id,
                    )
                    .await
                    {
                        println!("Failed to register commands in guild {}: {}", guild.id, e);
                    }
                }
                Ok(Data {})
            })
        })
        .build();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
