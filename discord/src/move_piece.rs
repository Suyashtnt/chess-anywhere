use crate::{error::CommandError, Context};
use error_stack::Result;

#[poise::command(slash_command)]
pub async fn r#move(ctx: Context<'_>) -> Result<(), CommandError> {
    todo!()
}
