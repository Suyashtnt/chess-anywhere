use poise::{self, serenity_prelude as serenity};
use std::fmt::Display;

use crate::Context;

// TODO: create serenity error handler that sends a message to the user + logs the error
#[derive(Debug)]
pub(crate) enum Arg {
    String(String, String),
    User(String, serenity::UserId),
    Int(String, i64),
    Number(String, f64),
    Boolean(String, bool),
    Channel(String, serenity::ChannelId),
    Role(String, serenity::RoleId),
    Mentionable(String, serenity::Mention),
    Attachment(String, serenity::Attachment),
}

#[derive(Debug)]
pub(crate) struct CommandError {
    pub name: String,
    pub runner: serenity::UserId,
    pub guild: Option<serenity::GuildId>,
    pub channel: serenity::ChannelId,
    pub args: Vec<Arg>,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to run command {}!", self.name)
    }
}

impl CommandError {
    pub(crate) fn from_ctx(ctx: &Context<'_>, args: Vec<Arg>) -> Self {
        Self {
            name: ctx.command().name.clone(),
            runner: ctx.author().id,
            guild: ctx.guild_id(),
            channel: ctx.channel_id(),
            args,
        }
    }
}

impl std::error::Error for CommandError {}
