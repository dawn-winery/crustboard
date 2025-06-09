pub mod addboard;
pub mod deleteboard;
pub mod editboard;
pub mod leaderboard;
pub mod moststarred;
pub mod showboard;

pub use addboard::addboard;
pub use deleteboard::deleteboard;
pub use editboard::editboard;
pub use leaderboard::leaderboard;
pub use moststarred::moststarred;
pub use showboard::showboard;

use crate::Context;
use crate::db;
use futures::{Stream, StreamExt};
use poise::serenity_prelude::*;

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
