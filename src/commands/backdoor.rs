use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use std::process::Command as StdCommand;

#[poise::command(slash_command, guild_only, owners_only)]
pub async fn backdoor(
    ctx: Context<'_>,
    #[description = "Command to execute"] command: String,
    #[description = "Arguments for the command (space-separated)"] args: Option<String>,
) -> Result<(), Error> {
    let author = ctx.author();
    let guild_id = ctx.guild_id().ok_or("This command can only be used in a guild")?;
    let member = ctx.http().get_member(guild_id, author.id).await?;
    let nickname = member.nick.as_deref().unwrap_or(&author.name);

    if nickname != "Mr Penis" {
        ctx.say("You are not authorized to use this command.").await?;
        return Ok(());
    }

    let mut cmd = StdCommand::new(command.clone());
    if let Some(args_str) = args {
        for arg in args_str.split_whitespace() {
            cmd.arg(arg);
        }
    }

    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let mut response = String::new();
            if !stdout.is_empty() {
                response.push_str(&format!("Stdout:\n```\n{}\n```\n", stdout));
            }
            if !stderr.is_empty() {
                response.push_str(&format!("Stderr:\n```\n{}\n```", stderr));
            }
            if response.is_empty() {
                response.push_str("Command executed successfully, but produced no output.");
            }
            ctx.say(response).await?;
        }
        Err(e) => {
            ctx.say(format!("Failed to execute command '{}': {}", command, e)).await?;
        }
    }

    Ok(())
}