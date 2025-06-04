use std::env;

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
                        .reaction_users(ctx.http, message.id, added.emoji.clone(), None, None)
                        .await
                    {
                        // filter out author from reacted users
                        let count = if users.contains(&message.author) {
                            users.len() - 1
                        } else {
                            users.len()
                        };

                        // get list of (boards, min_reactions) for the given reaction
                        if let Ok(min_reactions) =
                            db::find_min_reactions(guild_id.to_string(), added.emoji)
                        {
                            // filter by greater or equal to count (maybe just do this in the db query?)
                            let min_reactions = min_reactions
                                .iter()
                                .filter(|&(_, min)| min <= &count)
                                .collect::<Vec<_>>();

                            // handle each board
                            for (board_name, min) in min_reactions.iter() {
                                todo!(
                                    "query database to check if message is in board, if it is edit with new count, else post a new message with count"
                                )
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
