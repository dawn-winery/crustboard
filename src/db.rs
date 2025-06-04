use rusqlite::{Connection, Result, Error};
use serenity::model::channel::{Reaction, ReactionType};

const DB_NAME: &str = "settings.db"; 

pub fn create_db() -> Result<()> {
    if !std::path::Path::new(DB_NAME).exists() {
        let conn = Connection::open(DB_NAME)?;

        conn.execute(
            "CREATE TABLE guilds (
                guild_id TEXT PRIMARY KEY, 
                post_channel TEXT
            )", ()
        )?;

        conn.execute(
            "CREATE TABLE boards (
                board_id INT PRIMARY KEY AUTOINCREMENT,

                guild_id TEXT,
                name TEXT,
                reactions TEXT,
                min_reactions INT,

                FOREIGN KEY (guild_id) REFERENCES guilds(guild_id)
            )", ()
        )?;
    }

    Ok(())
}

pub fn to_csv(values: Vec<ReactionType>) -> String {
    values.into_iter().map(|x| x.to_string()).collect::<Vec<String>>().join(",")
}

pub fn from_csv(values: String) -> Vec<ReactionType> {
    values.split(",").map(|x| ReactionType::try_from(x).ok_or(ReactionType::Unicode("ඞ".to_string()))).collect()
}

pub fn add_guild(guild_id: String, post_channel: String) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    conn.execute(
        "INSERT OR INGORE INTO guilds (guild_id, post_channel) VALUES (?, ?)", 
        (guild_id, post_channel)
    )?;

    Ok(())
}

pub fn guild_exists(guild_id: String) -> bool {
    let Ok(conn) = Connection::open(DB_NAME) else { return false };
    let Ok(mut stmt) = conn.prepare("SELECT * FROM guilds WHERE guild_id = ?") else { return false };
    stmt.execute([guild_id]).is_ok()
}

pub fn set_channel(guild_id: String, channel_id: String) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    if guild_exists(guild_id.clone()) {
        conn.execute(
            "UPDATE guilds WHERE guild_id = ? SET post_channel = ?",
            (guild_id, channel_id)
        )?;
    } else {
        conn.execute(
            "INSERT INTO guilds (guild_id, post_channel) VALUES (?, ?)",
            (guild_id, channel_id)
        )?;
    }

    Ok(())
}

pub fn add_board(guild_id: String, name: String, reactions: Vec<ReactionType>, min_reactions: Option<u32>) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    if guild_exists(guild_id.clone()) {
        conn.execute(
            "INSERT OR INGORE INTO boards (guild_id, name, reactions, min_reactions) VALUES (?, ?, ?, ?)", 
            (guild_id, name, to_csv(reactions), min_reactions.unwrap_or(5))
        )?;
    }

    Ok(())
}

pub fn delete_board(guild_id: String, board_name: String) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;
    todo!();
}

pub fn get_reaction_types(guild_id: String) -> Result<()> {
    let conn = Connection::open(DB_NAME)?;

    if guild_exists(guild_id) {
        let mut stmt = conn.prepare("SELECT reactions FROM boards WHERE guild_id = ?")?;
        return stmt
            .query_map([guild_id], |row| from_csv(row.get(0).unwrap_or("N/A".to_string())))?
            .collect();
    }

    Err(Error::InvalidColumnName("Guild does not exist".to_string()))
}

