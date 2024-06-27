use crate::{error::CommandError, Context};
use error_stack::Result;

#[poise::command(slash_command)]
pub async fn new_game(ctx: Context<'_>) -> Result<(), CommandError> {
    todo!()
}

#[poise::command(slash_command)]
pub async fn new_external_game(ctx: Context<'_>) -> Result<(), CommandError> {
    todo!()
}
