use crate::{
    error::{Arg, CommandError},
    Context,
};
use error_stack::{Result, ResultExt};

#[poise::command(slash_command)]
pub async fn r#move(
    ctx: Context<'_>,
    #[description = "The move to make in SAN format"] r#move: String,
) -> Result<(), CommandError> {
    let move_to_make = r#move;
    let error = || {
        CommandError::from_ctx(
            &ctx,
            vec![Arg::String("move".to_string(), move_to_make.clone())],
        )
    };

    ctx.defer_ephemeral().await.change_context_lazy(error)?;

    let Some(player_platform) = ctx
        .data()
        .backend
        .find_player_discord(ctx.author().id)
        .await
    else {
        ctx.reply("You are not in a game!")
            .await
            .change_context_lazy(error)?;

        return Ok(());
    };

    todo!()
}
