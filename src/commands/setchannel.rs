use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue};

pub fn run(guild_id: String, options: &[ResolvedOption]) -> String {
    if let Some(ResolvedOption {value: ResolvedValue::Channel(channel), ..}) = options.first() {
        return if crate::db::set_channel(guild_id, channel.id.get().to_string()).is_ok() {
            format!("Channel {} has been set as the posting channel", channel.id)
        } else {
            format!("Failed to set channel")
        }
    }

    format!("No option? sussy :widesmile:")
}

pub fn register() -> CreateCommand {
    CreateCommand::new("setchannel")
        .description("Set the channel for the bot to post in")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Channel, "channel", "Channel to post in")
            .required(true)
        )
}
