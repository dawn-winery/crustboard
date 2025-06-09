use poise::serenity_prelude::*;
use rusqlite::{Connection, Result};

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

pub struct Message {
    pub user_id: String,
    pub source_id: String,
    pub dest_id: String,
    pub board_id: i64,
    pub reaction_count: i64,
}

pub fn create_db() -> Result<()> {
    if !std::path::Path::new(DB_NAME).exists() {
        let conn = get_connection()?;

        // min_reactions is the threshold for a message to be posted to the board
        // reactions holds csv of reaction IDs
        // dest_channel holds the channel ID of the channel that the message will be posted to
        //
        // user_id holds the user ID of the user that posted the message
        // source_id holds the message ID of the message that passed the reaction threshold
        // dest_id holds the message ID of the message that was posted to the board
        // board_id holds the ID of the board that the message reached the threshold for
        // reaction_count holds the number of reactions that is displayed on the destination message
        conn.execute_batch(
            "CREATE TABLE boards (
                board_id INTEGER PRIMARY KEY AUTOINCREMENT,

                guild_id TEXT,
                name TEXT,
                reactions TEXT,
                min_reactions INT,
                dest_channel TEXT
            )

            CREATE TABLE messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,

                user_id TEXT,
                source_id TEXT,
                dest_id TEXT,
                board_id INTEGER,
                reaction_count INTEGER,

                FOREIGN KEY(board_id) REFERENCES boards(board_id) ON DELETE CASCADE
            )",
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
        .filter_map(|x| ReactionType::try_from(x).ok())
        .collect()
}

pub fn add_board(
    guild_id: impl ToString,
    name: impl AsRef<str>,
    reactions: Vec<ReactionType>,
    min_reactions: Option<i64>,
    dest_channel: impl ToString,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO boards
            (guild_id, name, reactions, min_reactions, dest_channel)
            VALUES (?, ?, ?, ?, ?)",
        (
            guild_id.to_string(),
            name.as_ref(),
            to_csv(reactions),
            min_reactions.unwrap_or(5),
            dest_channel.to_string(),
        ),
    )?;

    Ok(())
}

pub fn delete_board(guild_id: impl ToString, board_name: impl AsRef<str>) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "DELETE FROM boards
            WHERE guild_id = ? AND name = ?",
        (guild_id.to_string(), board_name.as_ref()),
    )?;

    Ok(())
}

pub fn edit_board(
    guild_id: impl ToString,
    board_name: impl AsRef<str>,
    new_name: Option<String>,
    reactions: Option<Vec<ReactionType>>,
    min_reactions: Option<i64>,
    dest_channel: Option<impl ToString>,
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
            dest_channel.map(|c| c.to_string()),
            guild_id.to_string(),
            board_name.as_ref(),
        ),
    )?;

    Ok(())
}

pub fn get_board_names(guild_id: impl ToString) -> Result<Vec<String>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name
            FROM boards
            WHERE guild_id = ?",
    )?;

    stmt.query_map([guild_id.to_string()], |row| Ok(row.get(0)?))?
        .collect::<Result<Vec<String>>>()
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
    guild_id: impl ToString,
    board_name: impl ToString,
    user_id: impl ToString,
    source_id: impl ToString,
    dest_id: impl ToString,
    reaction_count: i64,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "INSERT INTO messages
            (board_id, user_id, source_id, dest_id, reaction_count)
            VALUES ((SELECT board_id FROM boards WHERE guild_id = ? AND name = ?), ?, ?, ?, ?)",
        (
            guild_id.to_string(),
            board_name.to_string(),
            user_id.to_string(),
            source_id.to_string(),
            dest_id.to_string(),
            reaction_count,
        ),
    )?;

    Ok(())
}

pub fn get_guild_messages(guild_id: impl ToString) -> Result<Vec<Message>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT user_id, source_id, dest_id, messages.board_id, reaction_count
            FROM messages
            JOIN boards ON messages.board_id = boards.board_id
            WHERE boards.guild_id = ?",
    )?;

    Ok(stmt
        .query_map([guild_id.to_string()], |row| {
            Ok(Message {
                user_id: row.get(0)?,
                source_id: row.get(1)?,
                dest_id: row.get(2)?,
                board_id: row.get::<usize, i64>(3)?,
                reaction_count: row.get(4)?,
            })
        })?
        .filter_map(|f| f.ok())
        .collect::<Vec<_>>())
}

pub fn get_board_messages(
    guild_id: impl ToString,
    board_name: impl ToString,
) -> Result<Vec<Message>> {
    let conn = get_connection()?;

    let board_id: i64 = conn.query_row(
        "SELECT board_id FROM boards WHERE guild_id = ? AND name = ?",
        (guild_id.to_string(), board_name.to_string()),
        |row| row.get(0),
    )?;

    let mut stmt = conn.prepare(
        "SELECT user_id, source_id, dest_id, reaction_count
            FROM messages
            WHERE board_id = ?",
    )?;

    Ok(stmt
        .query_map([board_id], |row| {
            Ok(Message {
                user_id: row.get(0)?,
                source_id: row.get(1)?,
                dest_id: row.get(2)?,
                board_id: board_id,
                reaction_count: row.get(3)?,
            })
        })?
        .filter_map(|f| f.ok())
        .collect::<Vec<Message>>())
}

// update reaction count of a message
pub fn update_message_reaction_count(
    guild_id: impl ToString,
    board_name: impl ToString,
    source_id: impl ToString,
    reaction_count: i64,
) -> Result<()> {
    let conn = get_connection()?;

    conn.execute(
        "UPDATE messages
            SET reaction_count = ?
            WHERE board_id = (SELECT board_id FROM boards WHERE guild_id = ? AND name = ?)
                AND source_id = ?",
        (
            reaction_count,
            guild_id.to_string(),
            board_name.to_string(),
            source_id.to_string(),
        ),
    )?;

    Ok(())
}

// get all boards for a guild
pub fn get_guild_boards(guild_id: impl ToString) -> Result<Vec<Board>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, reactions, min_reactions, dest_channel
            FROM boards
            WHERE guild_id = ?",
    )?;

    stmt.query_map([guild_id.to_string()], |row| {
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
pub fn get_board(guild_id: impl ToString, board_name: impl ToString) -> Result<Board> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, reactions, min_reactions, dest_channel
            FROM boards
            WHERE guild_id = ? AND name = ?",
    )?;

    stmt.query_row([guild_id.to_string(), board_name.to_string()], |row| {
        Ok(Board {
            name: row.get(0)?,
            reactions: row.get(1)?,
            min_reactions: row.get(2)?,
            dest_channel: row.get(3)?,
        })
    })
}

pub fn get_board_by_id(board_id: impl ToString) -> Result<Board> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT name, reactions, min_reactions, dest_channel
                FROM boards
                WHERE board_id = ?",
    )?;

    stmt.query_row([board_id.to_string()], |row| {
        Ok(Board {
            name: row.get(0)?,
            reactions: row.get(1)?,
            min_reactions: row.get(2)?,
            dest_channel: row.get(3)?,
        })
    })
}

pub fn get_board_user_reactions(
    guild_id: impl ToString,
    board_name: impl ToString,
) -> Result<Vec<(UserId, u64)>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT user_id, reaction_count
            FROM messages
            WHERE board_id = (SELECT board_id FROM boards WHERE guild_id = ? AND name = ?)",
    )?;

    let mut user_counts = std::collections::HashMap::new();

    stmt.query_map([guild_id.to_string(), board_name.to_string()], |row| {
        Ok((row.get::<usize, String>(0)?, row.get::<usize, u64>(1)?))
    })?
    .filter_map(|result| {
        result
            .map(|(user_id_str, count)| {
                user_id_str
                    .parse::<u64>()
                    .map(|user_id| (UserId::new(user_id), count))
                    .ok()
            })
            .ok()
            .flatten()
    })
    .for_each(|(user_id, count)| {
        *user_counts.entry(user_id).or_insert(0) += count;
    });

    Ok(user_counts.into_iter().collect::<Vec<(UserId, u64)>>())
}

pub fn get_guild_user_reactions(guild_id: impl ToString) -> Result<Vec<(UserId, u64)>> {
    let conn = get_connection()?;

    let mut stmt = conn.prepare(
        "SELECT user_id, reaction_count
            FROM messages
            WHERE board_id = (SELECT board_id FROM boards WHERE guild_id = ?)",
    )?;

    let mut user_counts = std::collections::HashMap::new();

    stmt.query_map([guild_id.to_string()], |row| {
        Ok((row.get::<usize, String>(0)?, row.get::<usize, u64>(1)?))
    })?
    .filter_map(|result| match result {
        Ok((user_id_str, count)) => match user_id_str.parse::<u64>() {
            Ok(id) => Some((UserId::new(id), count)),
            Err(_) => None,
        },
        Err(_) => None,
    })
    .for_each(|(user_id, count)| {
        *user_counts.entry(user_id).or_insert(0) += count;
    });

    Ok(user_counts.into_iter().collect::<Vec<(UserId, u64)>>())
}
