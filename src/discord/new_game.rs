use std::time::Duration;

use crate::{
    backend::{players::PlayerPlatform, CreateGameError},
    discord::{
        error::{Arg, Argument, CommandError},
        Context,
    },
    BACKEND_SERVICE, DISCORD_BOT_SERVICE,
};
use error_stack::{FutureExt, Result, ResultExt};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateMessage,
        EditMessage, Mentionable, Message, User,
    },
    CreateReply,
};

#[poise::command(
    slash_command,
    subcommands("discord", "discord_dm"),
    subcommand_required
)]
pub async fn new_game(_: Context<'_>) -> Result<(), CommandError> {
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
enum GameChoice {
    #[name = "White"]
    White,
    #[name = "Black"]
    Black,
    #[name = "Random"]
    Random,
    #[name = "Other decides! (Not implemented yet)"]
    TransferResponsibility,
}

#[poise::command(slash_command)]
#[tracing::instrument]
/// Play a game of chess with another user on this server
pub async fn discord(
    ctx: Context<'_>,
    #[description = "The user you want to play against"] other_user: User,
    #[description = "Which side do you want to play?"] side: GameChoice,
) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&ctx);

    let error_user = other_user.clone();
    let error_user = move || Argument(error_user.name.clone(), Arg::User(error_user.id.clone()));

    if other_user.bot {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("You can't challenge a bot!"),
        )
        .change_context_lazy(error)
        .attach(error_user)
        .await?;

        return Ok(());
    }

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("accept")
            .label("Accept")
            .style(ButtonStyle::Primary),
        CreateButton::new("decline")
            .label("Decline")
            .style(ButtonStyle::Danger),
    ])];

    let challenge_message = ctx
        .send(
            CreateReply::default()
                .content(format!(
                    "{}, you have been challenged to a game of chess by {}! Do you accept?",
                    other_user.mention(),
                    ctx.author().mention()
                ))
                .components(components),
        )
        .await
        .change_context_lazy(error)
        .attach_lazy(&error_user)?;

    let mut message = challenge_message
        .into_message()
        .await
        .change_context_lazy(error)?;

    let response = ComponentInteractionCollector::new(ctx)
        .author_id(other_user.id)
        .message_id(message.id)
        .timeout(Duration::from_secs(60))
        .await;

    match response {
        Some(response) => match response.data.custom_id == "accept" {
            true => {
                message
                    .edit(
                        ctx.http(),
                        EditMessage::default()
                            .content(format!(
                                "{} has accepted the challenge! The game will begin shortly.",
                                other_user.mention()
                            ))
                            .components(vec![]),
                    )
                    .change_context_lazy(error)
                    .attach_lazy(error_user)
                    .await?;

                start_game_both_discord(ctx, other_user, side, message).await?;
            }
            false => {
                message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content(format!(
                                "{} has declined the challenge. Maybe next time!",
                                other_user.mention()
                            ))
                            .components(vec![]),
                    )
                    .change_context_lazy(error)
                    .attach_lazy(error_user)
                    .await?
            }
        },
        None => {
            message
                .reply(
                    &ctx.http(),
                    format!("@silent {} took too long to respond", other_user.mention()),
                )
                .change_context_lazy(error)
                .change_context_lazy(error)
                .await?;
        }
    };

    Ok(())
}

async fn start_game_both_discord(
    ctx: Context<'_>,
    other_user: User,
    side: GameChoice,
    mut message: Message,
) -> Result<(), CommandError> {
    let error_user = other_user.clone();
    let error_user = move || Argument(error_user.name.clone(), Arg::User(error_user.id.clone()));

    let error = || CommandError::from_ctx(&ctx);

    let author = PlayerPlatform::Discord {
        user: ctx.author().clone(),
        game_message: message.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    let opponent = PlayerPlatform::Discord {
        user: other_user,
        game_message: message.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    let (white, black) = match side {
        GameChoice::White => (author, opponent),
        GameChoice::Black => (opponent, author),
        GameChoice::Random => {
            if rand::random() {
                (author, opponent)
            } else {
                (opponent, author)
            }
        }
        GameChoice::TransferResponsibility => {
            message
                .edit(
                    &ctx.http(),
                    EditMessage::default()
                        .content("Asking the other user to decide is not implemented yet!")
                        .components(vec![]),
                )
                .await
                .change_context_lazy(error)?;

            return Ok(());
        }
    };

    let backend = BACKEND_SERVICE.get().unwrap();
    let res = backend.create_game(white, black).await;

    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            let current_frame: &CreateGameError =
                err.frames().find_map(|frame| frame.downcast_ref()).unwrap();

            match current_frame {
                CreateGameError::PlayerInGame => {
                    message
                        .edit(
                            &ctx.http(),
                            EditMessage::default()
                                .content("One of y'all are already in a game!")
                                .components(vec![]),
                        )
                        .change_context_lazy(error)
                        .attach(error_user)
                        .await
                }
                _ => return Err(err.change_context(error())),
            }
        }
    }
}

#[poise::command(slash_command)]
#[tracing::instrument]
/// Challenge a user to a private game
pub async fn discord_dm(
    ctx: Context<'_>,
    #[description = "The user you want to play against"] other_user: User,
    #[description = "Which side do you want to play?"] side: GameChoice,
    #[description = "Do you also want to play in a DM?"] play_in_dm: bool,
) -> Result<(), CommandError> {
    let error_user = other_user.clone();
    let error_user = move || Argument(error_user.name.clone(), Arg::User(error_user.id.clone()));

    let error = || CommandError::from_ctx(&ctx);

    let mut message = if play_in_dm {
        ctx.send(
            CreateReply::default()
                .content("Challenge sending! Check your DMs for further info")
                .ephemeral(true),
        )
        .change_context_lazy(error)
        .attach_lazy(&error_user)
        .await?;

        ctx.author()
            .create_dm_channel(ctx.http())
            .await
            .change_context_lazy(error)
            .attach_lazy(&error_user)?
            .send_message(
                &ctx.http(),
                CreateMessage::default()
                    .content("Waiting for the other user to accept the challenge..."),
            )
            .change_context_lazy(error)
            .attach_lazy(&error_user)
            .await?
    } else {
        ctx.send(
            CreateReply::default().content("Waiting for the other user to accept the challenge..."),
        )
        .change_context_lazy(error)
        .attach_lazy(&error_user)
        .await?
        .into_message()
        .change_context_lazy(error)
        .attach_lazy(&error_user)
        .await?
    };

    let author = PlayerPlatform::Discord {
        user: ctx.author().clone(),
        game_message: message.clone(),
        http: ctx.serenity_context().http.clone(),
    };

    match DISCORD_BOT_SERVICE
        .get()
        .unwrap()
        .challenge_user_discord(&ctx.author().name, other_user.id)
        .change_context_lazy(error)
        .attach_lazy(&error_user)
        .await
    {
        Ok(Some(opponent)) => {
            let (white, black) = match side {
                GameChoice::White => (author, opponent),
                GameChoice::Black => (opponent, author),
                GameChoice::Random => {
                    if rand::random() {
                        (author, opponent)
                    } else {
                        (opponent, author)
                    }
                }
                GameChoice::TransferResponsibility => {
                    message
                        .edit(
                            &ctx.http(),
                            EditMessage::default()
                                .content("Asking the other user to decide is not implemented yet!")
                                .components(vec![]),
                        )
                        .await
                        .change_context_lazy(error)?;

                    return Ok(());
                }
            };

            let backend = BACKEND_SERVICE.get().unwrap();

            backend
                .create_game(white, black)
                .change_context_lazy(error)
                .attach_lazy(error_user)
                .await
        }
        Ok(None) => {
            message
                .edit(
                    ctx.http(),
                    EditMessage::default()
                        .content("The other user declined the challenge. Better luck next time!"),
                )
                .change_context_lazy(error)
                .attach_lazy(error_user)
                .await
        }
        Err(err) => {
            let current_frame: &CreateGameError =
                err.frames().find_map(|frame| frame.downcast_ref()).unwrap();

            match current_frame {
                CreateGameError::PlayerInGame => {
                    message
                        .edit(
                            &ctx.http(),
                            EditMessage::default()
                                .content("One of y'all are already in a game!")
                                .components(vec![]),
                        )
                        .change_context_lazy(error)
                        .attach(error_user)
                        .await
                }
                _ => return Err(err.change_context(error())),
            }
        }
    }
}
