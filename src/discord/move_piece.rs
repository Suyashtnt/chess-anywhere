use crate::{
    backend::chess::{ChessError, MoveStatus, SanArray},
    discord::{
        error::{Arg, Argument, CommandError},
        Context,
    },
    BACKEND_SERVICE,
};
use error_stack::{FutureExt, Result};
use poise::CreateReply;

#[poise::command(slash_command)]
pub async fn r#move(
    ctx: Context<'_>,
    #[description = "The move to make in SAN format"]
    #[autocomplete = "autocomplete_moves"]
    r#move: String,
) -> Result<(), CommandError> {
    let move_to_make = r#move;
    let error = || CommandError::from_ctx(&ctx);

    let backend = BACKEND_SERVICE.get().unwrap();
    let Some(player_platform) = backend.find_player_discord(ctx.author().id).await else {
        ctx.send(
            CreateReply::default()
                .content("You are not in a game!")
                .ephemeral(true),
        )
        .change_context_lazy(error)
        .await?;

        return Ok(());
    };

    ctx.defer_ephemeral().change_context_lazy(&error).await?;

    let outcome = backend.play_move(player_platform, &move_to_make).await;

    match outcome {
        Ok(last_move) => {
            match last_move {
                MoveStatus::Move(chess_move) => {
                    ctx.send(
                        CreateReply::default()
                            .content(format!("Move {} played successfully!", chess_move))
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                MoveStatus::Check => {
                    ctx.send(CreateReply::default().content("Checked!").ephemeral(true))
                        .change_context_lazy(error)
                        .await?;
                }
                MoveStatus::Checkmate => {
                    ctx.send(
                        CreateReply::default()
                            .content("Game over! Checkmate!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                MoveStatus::Stalemate => {
                    ctx.send(
                        CreateReply::default()
                            .content("Game over! Stalemate!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                MoveStatus::DrawOffer(_) => {
                    ctx.send(
                        CreateReply::default()
                            .content("Draw offer sent!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                MoveStatus::Draw => {
                    ctx.send(
                        CreateReply::default()
                            .content("Game over! Draw accepted!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                MoveStatus::GameStart => unreachable!(),
            };
        }
        Err(err) => {
            let current_frame: &ChessError =
                err.frames().find_map(|frame| frame.downcast_ref()).unwrap();

            match current_frame {
                ChessError::InvalidMove => {
                    ctx.send(
                        CreateReply::default()
                            .content("You played an invalid move!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                ChessError::NotYourTurn => {
                    ctx.send(
                        CreateReply::default()
                            .content("It's not your turn!")
                            .ephemeral(true),
                    )
                    .change_context_lazy(error)
                    .await?;
                }
                _ => {
                    return Err(err
                        .change_context(error())
                        .attach(Argument("move".to_string(), Arg::String(move_to_make))))
                }
            }
        }
    }
    Ok(())
}

async fn autocomplete_moves<'a>(ctx: Context<'_>, partial: &'a str) -> SanArray {
    let backend = BACKEND_SERVICE.get().unwrap();

    let Some(player_platform) = backend.find_player_discord(ctx.author().id).await else {
        return SanArray::new_const();
    };

    backend
        .get_moves(&player_platform)
        .await
        .into_iter()
        .filter(|san| san.starts_with(partial))
        .collect()
}
