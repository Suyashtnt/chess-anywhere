use poise::{
    self,
    serenity_prelude::{self as serenity, Mentionable},
};
use std::fmt::{self, Display};

use crate::Context;

#[derive(Debug)]
pub(crate) enum Arg {
    String(String),
    User(serenity::UserId),
    Int(i64),
    Number(f64),
    Boolean(bool),
    Channel(serenity::ChannelId),
    Role(serenity::RoleId),
    Mentionable(serenity::Mention),
    Attachment(serenity::Attachment),
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::String(s) => write!(f, "{}", s),
            Arg::User(u) => write!(f, "{}", u.mention()),
            Arg::Int(i) => write!(f, "{}", i),
            Arg::Number(n) => write!(f, "{}", n),
            Arg::Boolean(b) => write!(f, "{}", b),
            Arg::Channel(c) => write!(f, "{}", c.mention()),
            Arg::Role(r) => write!(f, "{}", r.mention()),
            Arg::Mentionable(m) => write!(f, "{}", m.mention()),
            Arg::Attachment(a) => write!(f, "{}", a.filename),
        }
    }
}

#[derive(Debug)]
pub struct Argument(pub String, pub Arg);

#[derive(Debug)]
pub(crate) struct CommandError {
    pub name: String,
    pub runner: serenity::UserId,
    pub guild: Option<serenity::GuildId>,
    pub channel: serenity::ChannelId,
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to run command {}!", self.name)
    }
}

impl CommandError {
    pub(crate) fn from_ctx(ctx: &Context<'_>) -> Self {
        Self {
            name: ctx.command().name.clone(),
            runner: ctx.author().id,
            guild: ctx.guild_id(),
            channel: ctx.channel_id(),
        }
    }
}

impl std::error::Error for CommandError {}
