use rusqlite::{Connection, Error, Result};
use serenity::model::channel::{Reaction, ReactionType};

const DB_NAME: &str = "settings.db";

fn get_connection() -> Result<Connection> {
    Connection::open(DB_NAME)
}

pub fn create_db() -> Result<()> {
    if !std::path::Path::new(DB_NAME).exists() {
        let conn = get_connection()?;

        // min_reactions is the threshold for a message to be posted to the board
        // reactions holds csv of reaction IDs
        // dest_channel holds the channel ID of the channel that the message will be posted to
        conn.execute(
            "CREATE TABLE boards (
                board_id INTEGER PRIMARY KEY AUTOINCREMENT,

                guild_id TEXT,
                name TEXT,
                reactions TEXT,
                min_reactions INT
                dest_channel TEXT,
            )",
            (),
        )?;

        // source_id holds the message ID of the message that passed the reaction threshold
        // dest_id holds the message ID of the message that was posted to the board
        // board_id holds the ID of the board that the message reached the threshold for
        // reaction_count holds the number of reactions that is displayed on the destination message
        conn.execute(
            "CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,

                source_id TEXT,
                dest_id TEXT,
                board_id INTEGER,
                reaction_count INTEGER,

                FOREIGN KEY(board_id) REFERENCES boards(board_id)
            )",
            (),
        )?;
    }

    Ok(())
}

pub fn to_csv(values: Vec<ReactionType>) -> String {
    values
        .into_iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

pub fn from_csv(values: String) -> Vec<ReactionType> {
    values
        .split(",")
        .filter_map(|x| match ReactionType::try_from(x) {
            Ok(reaction) => Some(reaction),
            Err(_) => Some(ReactionType::Unicode("ඞ".to_string())),
        })
        .collect()
}

pub fn add_board(
    guild_id: String,
    name: String,
    reactions: Vec<ReactionType>,
    min_reactions: Option<u32>,
    dest_channel: String,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO boards (guild_id, name, reactions, min_reactions, dest_channel) VALUES (?, ?, ?, ?, ?)",
        (guild_id, name, to_csv(reactions), min_reactions.unwrap_or(5), dest_channel),
    )?;

    Ok(())
}

pub fn delete_board(guild_id: String, board_name: String) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "DELETE FROM boards WHERE guild_id = ? AND name = ?",
        (guild_id, board_name),
    )?;

    Ok(())
}

// get board name and minimum reactions given the boards contain passed ReactionType
pub fn find_min_reactions(
    guild_id: String,
    reaction: ReactionType,
) -> Result<Vec<(String, usize)>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, min_reactions FROM boards WHERE guild_id = ? AND reactions LIKE ?",
    )?;

    stmt.query_map([guild_id, format!("%{}%", reaction)], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?
    .collect::<Result<Vec<(String, usize)>>>()
}
