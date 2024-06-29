use crate::{
    error::{Arg, CommandError},
    Context,
};
use backend::chess::ChessError;
use error_stack::{Result, ResultExt};
use poise::CreateReply;

#[poise::command(slash_command)]
pub async fn r#move(
    ctx: Context<'_>,
    #[description = "The move to make in SAN format"] r#move: String,
) -> Result<(), CommandError> {
    let move_to_make = r#move;
    let error_move = move_to_make.clone();
    let error = || {
        CommandError::from_ctx(
            &ctx,
            vec![Arg::String("move".to_string(), error_move.clone())],
        )
    };

    let Some(player_platform) = ctx
        .data()
        .backend
        .find_player_discord(ctx.author().id)
        .await
    else {
        ctx.send(
            CreateReply::default()
                .content("You are not in a game!")
                .ephemeral(true),
        )
        .await
        .change_context_lazy(error)?;

        return Ok(());
    };

    let outcome = ctx
        .data()
        .backend
        .play_move(player_platform, move_to_make)
        .await;

    match outcome {
        Ok(last_move) => {
            match last_move {
                backend::chess::MoveStatus::Move(chess_move) => {
                    ctx.send(
                        CreateReply::default()
                            .content(format!("Move {} played successfully!", chess_move))
                            .ephemeral(true),
                    )
                    .await
                    .change_context_lazy(error)?;
                }
                backend::chess::MoveStatus::Check => {
                    ctx.send(CreateReply::default().content("Checked!").ephemeral(true))
                        .await
                        .change_context_lazy(error)?;
                }
                backend::chess::MoveStatus::Checkmate => {
                    ctx.send(
                        CreateReply::default()
                            .content("Game over! Checkmate!")
                            .ephemeral(true),
                    )
                    .await
                    .change_context_lazy(error)?;
                }
                backend::chess::MoveStatus::Stalemate => {
                    ctx.send(
                        CreateReply::default()
                            .content("Game over! Stalemate!")
                            .ephemeral(true),
                    )
                    .await
                    .change_context_lazy(error)?;
                }
                backend::chess::MoveStatus::GameStart => unreachable!(),
            };
        }
        Err(err) => {
            let current_frame: &ChessError =
                err.frames().find_map(|frame| frame.downcast_ref()).unwrap();

            match current_frame {
                ChessError::InvalidMove => {
                    ctx.say("You played an invalid move!")
                        .await
                        .change_context_lazy(error)?;
                }
                ChessError::NotYourTurn => {
                    ctx.say("It is not your turn!")
                        .await
                        .change_context_lazy(error)?;
                }
                _ => return Err(err.change_context(error())),
            }
        }
    }
    Ok(())
}
