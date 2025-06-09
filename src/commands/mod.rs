pub mod addboard;
pub mod deleteboard;
pub mod editboard;
pub mod leaderboard;
pub mod moststarred;
pub mod random;
pub mod showboard;

pub use addboard::addboard;
pub use deleteboard::deleteboard;
pub use editboard::editboard;
pub use leaderboard::leaderboard;
pub use moststarred::moststarred;
pub use random::random;
pub use showboard::showboard;

use crate::{Context, Error, db};
use futures::{Stream, StreamExt};
use poise::{CreateReply, serenity_prelude::*};

pub fn parse_reactions(reactions: String) -> Vec<ReactionType> {
    let mut parsed_reactions = Vec::new();

    // parse space-separated reactions
    for reaction in reactions.split_whitespace() {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }

    // search by custom emoji format <:name:id>
    let split_reactions = reactions.split(">").collect::<Vec<&str>>();
    let len = split_reactions.len();
    let reactions = split_reactions
        .into_iter()
        .take(len - 1)
        .map(|r| r.trim().to_owned() + ">")
        .collect::<Vec<String>>();

    // parse ReactionType from reactions
    for reaction in reactions {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }

    // remove duplicates
    parsed_reactions.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    parsed_reactions.dedup_by(|a, b| a.to_string() == b.to_string());
    parsed_reactions
}

pub async fn autocomplete_board_names<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let guild_id = ctx.guild_id().expect("Guild ID not found");

    let board_names = db::get_board_names(guild_id).unwrap_or_default();

    futures::stream::iter(board_names)
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string())
}

pub async fn create_reply(ctx: Context<'_>, message: Message) -> Result<CreateReply, Error> {
    Ok(ctx.reply_builder(CreateReply {
        content: Some(message.content),
        embeds: message
            .embeds
            .iter()
            .map(|e| {
                let mut embed = CreateEmbed::new();
                if let Some(author) = &e.author {
                    embed = embed.author(CreateEmbedAuthor::from(author.clone()));
                }
                if let Some(color) = e.colour {
                    embed = embed.color(color);
                }
                if let Some(description) = &e.description {
                    embed = embed.description(description.clone());
                }
                if let Some(footer) = &e.footer {
                    embed = embed.footer(CreateEmbedFooter::from(footer.clone()));
                }
                if let Some(image) = &e.image {
                    embed = embed.image(image.url.clone());
                }
                if let Some(timestamp) = e.timestamp {
                    embed = embed.timestamp(timestamp);
                }
                if let Some(url) = &e.url {
                    embed = embed.url(url);
                }

                embed
            })
            .collect(),
        attachments: {
            let mut attachments = Vec::new();
            for attachment in message.attachments {
                attachments.push(CreateAttachment::url(ctx.http(), attachment.url.as_str()).await?);
            }
            attachments
        },
        ephemeral: None,
        components: None,
        allowed_mentions: None,
        reply: false,
        __non_exhaustive: (),
    }))
}
