use crate::db;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(slash_command, guild_only)]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "The name of the board to display"] name: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command can only be used in a guild")?;

    // fetch either specified board or all boards' data
    let board_data = {
        if let Some(ref name) = name {
            db::get_board_user_reactions(guild_id.to_string(), name.clone())
        } else {
            db::get_guild_user_reactions(guild_id.to_string())
        }
    };

    match board_data {
        Ok(mut data) => {
            if data.is_empty() {
                let message = match name {
                    Some(board_name) => format!("No data found for board '{}'", board_name),
                    None => "No leaderboard data found for this server".to_string(),
                };
                ctx.say(message).await?;
                return Ok(());
            }

            // sort desc by reaction count
            data.sort_by(|a, b| b.1.cmp(&a.1));

            // create pages of 10 results each
            let pages = create_leaderboard_pages(&data);

            if pages.is_empty() {
                ctx.say("Failed to create leaderboard pages").await?;
                return Ok(());
            }

            let title = match name {
                Some(ref board_name) => format!("{} Leaderboard", board_name),
                None => "Server Leaderboard".to_string(),
            };

            // use pagination if more than one page, otherwise just send the single page
            if pages.len() > 1 {
                paginate_leaderboard(ctx, title, &pages).await?;
            } else {
                let embed = serenity::CreateEmbed::new()
                    .title(title)
                    .description(&pages[0])
                    .color(0x00ff00);

                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            }
        }
        Err(err) => {
            ctx.say(format!("Error fetching leaderboard data: {}", err))
                .await?;
        }
    }

    Ok(())
}

fn create_leaderboard_pages(data: &[(serenity::UserId, u64)]) -> Vec<String> {
    const ENTRIES_PER_PAGE: usize = 10;
    let mut pages = Vec::new();

    for (page_num, chunk) in data.chunks(ENTRIES_PER_PAGE).enumerate() {
        let mut page_content = String::new();

        for (index, (user_id, count)) in chunk.iter().enumerate() {
            let rank = page_num * ENTRIES_PER_PAGE + index + 1;

            page_content.push_str(&format!(
                "**#{}** <@{}> • {} reactions\n",
                rank, user_id, count
            ));
        }

        // footer
        let total_pages = (data.len() + ENTRIES_PER_PAGE - 1) / ENTRIES_PER_PAGE;
        page_content.push_str(&format!("\n*Page {} of {}*", page_num + 1, total_pages));

        pages.push(page_content);
    }

    pages
}

async fn paginate_leaderboard(
    ctx: Context<'_>,
    title: String,
    pages: &[String],
) -> Result<(), Error> {
    // Define unique identifiers for navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // send initial embed with first page
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id)
                .emoji('◀')
                .label("Previous"),
            serenity::CreateButton::new(&next_button_id)
                .emoji('▶')
                .label("Next"),
        ]);

        let embed = serenity::CreateEmbed::new()
            .title(title.clone())
            .description(&pages[0])
            .color(0x00ff00);

        poise::CreateReply::default()
            .embed(embed)
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // handle navigation interactions
    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
        .await
    {
        // update current page based on button pressed
        if press.data.custom_id == next_button_id {
            current_page = (current_page + 1) % pages.len();
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            continue;
        }

        // update the message with new page content
        let embed = serenity::CreateEmbed::new()
            .title(title.clone())
            .description(&pages[current_page])
            .color(0x00ff00);

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;
    }

    Ok(())
}
