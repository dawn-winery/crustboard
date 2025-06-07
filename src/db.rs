use rusqlite::{Connection, Result};
use serenity::model::channel::ReactionType;

const DB_NAME: &str = "settings.db";

fn get_connection() -> Result<Connection> {
    Connection::open(DB_NAME)
}

pub struct Board {
    pub name: String,
    pub reactions: String,
    pub min_reactions: i32,
    pub dest_channel: String,
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
                min_reactions INT,
                dest_channel TEXT
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

                FOREIGN KEY(board_id) REFERENCES boards(board_id) ON DELETE CASCADE
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
    min_reactions: Option<i64>,
    dest_channel: String,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO boards
            (guild_id, name, reactions, min_reactions, dest_channel)
            VALUES (?, ?, ?, ?, ?)",
        (
            guild_id,
            name,
            to_csv(reactions),
            min_reactions.unwrap_or(5),
            dest_channel,
        ),
    )?;

    Ok(())
}

pub fn delete_board(guild_id: String, board_name: String) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "DELETE FROM boards
            WHERE guild_id = ? AND name = ?",
        (guild_id, board_name),
    )?;

    Ok(())
}

pub fn edit_board(
    guild_id: String,
    board_name: String,
    new_name: Option<String>,
    reactions: Option<Vec<ReactionType>>,
    min_reactions: Option<i64>,
    dest_channel: Option<String>,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE boards
            SET name = COALESCE(?, name),
                reactions = COALESCE(?, reactions),
                min_reactions = COALESCE(?, min_reactions),
                dest_channel = COALESCE(?, dest_channel)
            WHERE guild_id = ? AND name = ?",
        (
            new_name,
            reactions.map(|r| to_csv(r)),
            min_reactions,
            dest_channel,
            guild_id,
            board_name,
        ),
    )?;

    Ok(())
}

// get board name, min_reactions and dest_channel given the boards contain passed ReactionType
pub fn find_min_reactions(
    guild_id: String,
    reaction: ReactionType,
) -> Result<Vec<(String, usize, String)>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, min_reactions, dest_channel
            FROM boards
            WHERE guild_id = ? AND reactions LIKE ?",
    )?;

    stmt.query_map([guild_id, format!("%{}%", reaction)], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect::<Result<Vec<(String, usize, String)>>>()
}

pub fn get_message_dest(guild_id: String, source_id: String) -> Result<String> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT messages.dest_id
            FROM messages
            JOIN boards ON messages.board_id = boards.board_id
            WHERE boards.guild_id = ? AND messages.source_id = ?",
    )?;

    let message_id: String = stmt.query_row([guild_id, source_id], |row| row.get(0))?;

    Ok(message_id)
}

// add a message to the messages table
pub fn add_message(
    guild_id: String,
    board_name: String,
    source_id: String,
    dest_id: String,
    reaction_count: i64,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO messages
            (board_id, source_id, dest_id, reaction_count)
            VALUES ((SELECT board_id FROM boards WHERE guild_id = ? AND name = ?), ?, ?, ?)",
        (guild_id, board_name, source_id, dest_id, reaction_count),
    )?;

    Ok(())
}

// update reaction count of a message
pub fn update_message_reaction_count(
    guild_id: String,
    board_name: String,
    source_id: String,
    reaction_count: i64,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE messages
            SET reaction_count = ?
            WHERE board_id = (SELECT board_id FROM boards WHERE guild_id = ? AND name = ?)
                AND source_id = ?",
        (reaction_count, guild_id, board_name, source_id),
    )?;

    Ok(())
}

// get all boards for a guild
pub fn get_guild_boards(guild_id: String) -> Result<Vec<Board>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, reactions, min_reactions, dest_channel
            FROM boards
            WHERE guild_id = ?",
    )?;

    stmt.query_map([guild_id], |row| {
        Ok(Board {
            name: row.get(0)?,
            reactions: row.get(1)?,
            min_reactions: row.get(2)?,
            dest_channel: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<Board>>>()
}

// get a board by name
pub fn get_board(guild_id: String, board_name: String) -> Result<Board> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, reactions, min_reactions, dest_channel
            FROM boards
            WHERE guild_id = ? AND name = ?",
    )?;

    stmt.query_row([guild_id, board_name], |row| {
        Ok(Board {
            name: row.get(0)?,
            reactions: row.get(1)?,
            min_reactions: row.get(2)?,
            dest_channel: row.get(3)?,
        })
    })
}
