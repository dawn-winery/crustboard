use std::env;

use serenity::all::{Color, CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage, MessageId};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::{application::Interaction, channel::Reaction, gateway::Ready};
use serenity::prelude::*;

pub mod db;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // function is called when a reaction is added to a message
    async fn reaction_add(&self, ctx: Context, added: Reaction) {
        match added.message(&ctx.http).await {
            Ok(message) => {
                let guild_id = added.guild_id.unwrap();

                // check if added reaction type is in boards
                let users = message
                    .channel_id
                    .reaction_users(
                        ctx.clone().http,
                        message.id,
                        added.emoji.clone(),
                        None,
                        None,
                    )
                    .await
                    .expect("Failed to fetch reaction users");

                // filter out author from reacted users
                let count = if users.contains(&message.author) {
                    users.len() - 1
                } else {
                    users.len()
                };

                // get list of (boards, min_reactions) for the given reaction
                let min_reactions = db::find_min_reactions(guild_id.to_string(), added.emoji)
                    .expect("Failed to fetch min reactions");

                // filter by greater or equal to count (maybe just do this in the db query?)
                let min_reactions = min_reactions
                    .iter()
                    .filter(|&(_, min, _)| min <= &count)
                    .collect::<Vec<_>>();

                // get all channels
                let guild_channels = ctx
                    .http()
                    .get_channels(guild_id)
                    .await
                    .expect("Failed to fetch channels");

                // handle each board
                for (board_name, _, dest_channel_id) in min_reactions.iter() {
                    // query database to check if message is in board
                    match db::get_message_dest(guild_id.to_string(), message.id.to_string()) {
                        Ok(dest_id) => {
                            // get matching channel
                            let dest_channel = guild_channels
                                .iter()
                                .find(|channel| {
                                    channel.id.to_string() == dest_channel_id.to_string()
                                })
                                .expect("No matching channel found");

                            // create updated message with new count
                            let edit_message = EditMessage::new().content(format!(
                                "{} **| {} Reactions |** <#{}> **({})**",
                                board_name, count, message.channel_id, message.author
                            ));

                            // edit message
                            if let Err(err) = ctx
                                .http()
                                .edit_message(
                                    dest_channel.id,
                                    MessageId::new(
                                        dest_id.parse::<u64>().expect("Failed to parse message ID"),
                                    ),
                                    &edit_message,
                                    Vec::new(),
                                )
                                .await
                            {
                                println!("Error editing message: {}", err);
                            }

                            // save to db
                            if let Err(err) = db::update_message_reaction_count(
                                guild_id.to_string(),
                                board_name.to_string(),
                                dest_id,
                                count as i64,
                            ) {
                                println!("Error updating message reaction count: {}", err);
                            }
                        }
                        Err(rusqlite::Error::QueryReturnedNoRows) => {
                            println!("No message found in database, posting new message");

                            // post a new message with count, save to db
                            let mut dest_message = CreateMessage::new().content(format!(
                                "{} **| {} Reactions |** <#{}> **({})**",
                                board_name, count, message.channel_id, message.author
                            ));

                            // handle replies to messages
                            if let Some(referenced_message) = message.referenced_message.clone() {
                                // create an embed with the referenced message
                                dest_message = dest_message.add_embed({
                                    let mut embed = CreateEmbed::new()
                                        .author(
                                            CreateEmbedAuthor::new(format!(
                                                "Replying to {}",
                                                referenced_message.author.name
                                            ))
                                            .url(referenced_message.link())
                                            .icon_url(
                                                referenced_message
                                                    .author
                                                    .avatar_url()
                                                    .unwrap_or(String::new()),
                                            ),
                                        )
                                        .description(referenced_message.content.to_string())
                                        .timestamp(referenced_message.timestamp);

                                    for attachment in &referenced_message.attachments {
                                        embed = embed.image(attachment.url.to_string());
                                    }

                                    embed
                                });

                                // add attachments
                                if referenced_message.attachments.len() > 1 {
                                    for attachment in &referenced_message.attachments {
                                        dest_message = dest_message.add_embed(
                                            CreateEmbed::new()
                                                .image(attachment.url.to_string())
                                                .color(Color::from_rgb(0x1d, 0xa0, 0xf2)),
                                        );
                                    }
                                }

                                // add embeds
                                for embed in referenced_message.embeds {
                                    dest_message = dest_message.add_embed(
                                        CreateEmbed::from(embed)
                                            .color(Color::from_rgb(0x63, 0x63, 0xff)),
                                    );
                                }
                            }

                            // create an embed with the author's message
                            dest_message = dest_message.add_embed({
                                let mut embed = CreateEmbed::new()
                                    .author(
                                        CreateEmbedAuthor::new(&message.author.name)
                                            .url(message.link())
                                            .icon_url(
                                                message
                                                    .author
                                                    .avatar_url()
                                                    .unwrap_or(String::new()),
                                            ),
                                    )
                                    .description(message.content.to_string())
                                    .color(Color::from_rgb(0xff, 0xe1, 0x9c))
                                    .timestamp(message.timestamp);

                                for attachment in &message.attachments {
                                    embed = embed.image(attachment.url.to_string());
                                }

                                embed
                            });

                            // add attachments
                            if message.attachments.len() > 1 {
                                for attachment in &message.attachments {
                                    dest_message = dest_message.add_embed(
                                        CreateEmbed::new()
                                            .image(attachment.url.to_string())
                                            .color(Color::from_rgb(0x1d, 0xa0, 0xf2)),
                                    );
                                }
                            }

                            // add embeds
                            for embed in message.embeds.clone() {
                                dest_message = dest_message.add_embed(
                                    CreateEmbed::from(embed)
                                        .color(Color::from_rgb(0x63, 0x63, 0xff)),
                                );
                            }

                            // get channel matching dest_channel_id
                            match guild_channels
                                .iter()
                                .find(|channel| &channel.id.to_string() == dest_channel_id)
                            {
                                Some(channel) => {
                                    // send message to destination channel
                                    match channel.send_message(&ctx.http, dest_message).await {
                                        Ok(dest_msg) => {
                                            // save to database
                                            if let Err(err) = db::add_message(
                                                guild_id.to_string(),
                                                board_name.to_string(),
                                                message.id.to_string(),
                                                dest_msg.id.to_string(),
                                                count as i64,
                                            ) {
                                                println!(
                                                    "Failed to save message to database: {}",
                                                    err
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            println!("Failed to send message: {}", err);
                                        }
                                    }
                                }
                                None => {
                                    println!("Destination channel not found");
                                }
                            }
                        }
                        Err(err) => {
                            println!("Failed to send message: {}", err);
                        }
                    }
                }
            }
            Err(e) => println!("{}", e),
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "addboard" => {
                    if let Some(guild_id) = command.guild_id {
                        Some(commands::addboard::run(
                            guild_id.to_string(),
                            &command.data.options(),
                        ))
                    } else {
                        None
                    }
                }
                "deleteboard" => {
                    if let Some(guild_id) = command.guild_id {
                        Some(commands::deleteboard::run(
                            guild_id.to_string(),
                            &command.data.options(),
                        ))
                    } else {
                        None
                    }
                }
                "showboard" => {
                    if let Some(guild_id) = command.guild_id {
                        Some(commands::showboard::run(
                            guild_id.to_string(),
                            &command.data.options(),
                        ))
                    } else {
                        None
                    }
                }
                "editboard" => {
                    if let Some(guild_id) = command.guild_id {
                        Some(commands::editboard::run(
                            guild_id.to_string(),
                            &command.data.options(),
                        ))
                    } else {
                        None
                    }
                }

                _ => Some("unimplemented".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        db::create_db().expect("Failed to create database");

        // set commands in each guild
        for guild in ready.guilds {
            guild
                .id
                .set_commands(
                    &ctx.http,
                    vec![
                        commands::addboard::register(),
                        commands::showboard::register(),
                        commands::deleteboard::register(),
                        commands::editboard::register(),
                    ],
                )
                .await
                .expect("Failed to set guild commands");
        }

        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
