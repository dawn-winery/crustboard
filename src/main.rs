use std::env;

use serenity::all::{Channel, CreateEmbed, CreateEmbedAuthor, CreateMessage, EmbedAuthor};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::{
    application::{Command, Interaction},
    channel::{Message, Reaction},
    gateway::Ready,
};
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
                if let Some(guild_id) = added.guild_id {
                    // check if added reaction type is in boards
                    if let Ok(users) = message
                        .channel_id
                        .reaction_users(
                            ctx.clone().http,
                            message.id,
                            added.emoji.clone(),
                            None,
                            None,
                        )
                        .await
                    {
                        // filter out author from reacted users
                        let count = if users.contains(&message.author) {
                            users.len() - 1
                        } else {
                            users.len()
                        };
                        println!("got {count} reactions");

                        // get list of (boards, min_reactions) for the given reaction
                        if let Ok(min_reactions) =
                            db::find_min_reactions(guild_id.to_string(), added.emoji)
                        {
                            // filter by greater or equal to count (maybe just do this in the db query?)
                            let min_reactions = min_reactions
                                .iter()
                                .filter(|&(_, min, _)| min <= &count)
                                .collect::<Vec<_>>();

                            println!("min_reactions: {:?}", min_reactions);

                            // handle each board
                            for (board_name, _, dest_channel_id) in min_reactions.iter() {
                                // query database to check if message is in board
                                match db::get_message_dest(
                                    guild_id.to_string(),
                                    message.id.to_string(),
                                ) {
                                    Ok(dest_id) => {
                                        // edit with new count
                                        todo!("fetch message from db, edit message with new count")
                                    }
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                                        println!(
                                            "No message found in database, posting new message"
                                        );

                                        // post a new message with count, save to db
                                        let mut dest_message =
                                            CreateMessage::new().content(format!(
                                                "**{} {} | {} ({})**",
                                                board_name,
                                                count,
                                                message.channel_id,
                                                message.author
                                            ));

                                        // handle replies to messages
                                        if let Some(referenced_message) =
                                            message.referenced_message.clone()
                                        {
                                            // create an embed with the referenced message
                                            dest_message = dest_message.add_embed(
                                                CreateEmbed::new()
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
                                                    .description(
                                                        referenced_message.content.to_string(),
                                                    )
                                                    .timestamp(referenced_message.timestamp),
                                            );
                                        }

                                        // create an embed with the author's message
                                        dest_message = dest_message.add_embed(
                                            CreateEmbed::new()
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
                                                .timestamp(message.timestamp),
                                        );

                                        // get all channels
                                        match ctx.http().get_channels(guild_id).await {
                                            Ok(guild_channels) => {
                                                // get channel matching dest_channel_id
                                                match guild_channels.iter().find(|channel| {
                                                    &channel.id.to_string() == dest_channel_id
                                                }) {
                                                    Some(channel) => {
                                                        // send message to destination channel
                                                        match channel
                                                            .send_message(&ctx.http, dest_message)
                                                            .await
                                                        {
                                                            Ok(msg) => {
                                                                todo!("Save to database")
                                                            }
                                                            Err(err) => {
                                                                println!(
                                                                    "Failed to send message: {}",
                                                                    err
                                                                );
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        println!("Destination channel not found");
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                println!(
                                                    "Could not find destination channel: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error getting message destination: {}", e);
                                    }
                                }
                            }
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
        Command::create_global_command(&ctx.http, commands::addboard::register())
            .await
            .unwrap_or_else(|why| {
                panic!("Cannot register command: {why}");
            });

        db::create_db().expect("Failed to create database");

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
