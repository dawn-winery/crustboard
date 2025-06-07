use serenity::all::ReactionType;

pub mod addboard;
pub mod deleteboard;
pub mod editboard;
pub mod showboard;

// find all occurances of the pattern <:text:emoji_id> or unicode emoji
pub fn parse_reactions(reactions: String) -> Vec<ReactionType> {
    let mut parsed_reactions = Vec::new();

    // really bad implementation
    for reaction in reactions.split_whitespace() {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }
    let split_reactions = reactions.split(">").collect::<Vec<&str>>();
    let len = split_reactions.len();
    let reactions = split_reactions
        .into_iter()
        .take(len - 1)
        .map(|r| r.trim().to_owned() + ">")
        .collect::<Vec<String>>();
    for reaction in reactions {
        if let Ok(reaction) = ReactionType::try_from(reaction) {
            parsed_reactions.push(reaction);
        }
    }

    // filter out duplicates by converting each value back to string
    parsed_reactions.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
    parsed_reactions.dedup_by(|a, b| a.to_string() == b.to_string());
    parsed_reactions
}
