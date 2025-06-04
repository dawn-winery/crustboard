use std::env;

use serenity::async_trait;
use serenity::model::{
    channel::{Message, Reaction},
    application::{Command, Interaction},
    gateway::Ready
};
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::prelude::*;

pub mod db;

mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, added: Reaction) {
        match added.message(&ctx.http).await {
            Ok(message) => {
                if let Some(guild_id) = added.guild_id {
                    // check if added reaction type is in boards


                    if let Ok(users) = message.channel_id.reaction_users(ctx.http, message.id, added.emoji, None, None).await {
                        // filter out author from reacted users
                        let count = users
                            .into_iter()
                            .filter(|x| *x != message.author)
                            .count();


                    }
                }
            },
            Err(e) => println!("{}", e)
        }
    }
    
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "setchannel" => {
                    if let Some(guild_id) = command.guild_id {
                        Some(commands::setchannel::run(guild_id.to_string(), &command.data.options()))
                    } else {
                        None
                    }
                },
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
        let guild_command =
            Command::create_global_command(&ctx.http, commands::setchannel::register())
                .await;

        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
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
