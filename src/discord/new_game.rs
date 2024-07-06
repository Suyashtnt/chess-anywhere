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
        EditMessage, Mentionable, User,
    },
    CreateReply,
};
use shakmaty::Color;

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

    if other_user.id == ctx.author().id {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("You can't challenge yourself!"),
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

                let author = PlayerPlatform::Discord {
                    user: ctx.author().clone(),
                    game_message: message.clone(),
                    http: ctx.serenity_context().http.clone(),
                };

                let opponent = PlayerPlatform::Discord {
                    user: other_user.clone(),
                    game_message: message.clone(),
                    http: ctx.serenity_context().http.clone(),
                };

                start_game_both_discord(ctx, side, author, opponent).await?;
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

#[poise::command(slash_command)]
#[tracing::instrument]
/// Challenge a user to a private game in their DMs
pub async fn discord_dm(
    ctx: Context<'_>,
    #[description = "The user you want to play against"] other_user: User,
    #[description = "Which side do you want to play?"] side: GameChoice,
    #[description = "Do you also want to play in a DM?"] play_in_dm: bool,
) -> Result<(), CommandError> {
    let error_user = other_user.clone();
    let error_user = move || Argument(error_user.name.clone(), Arg::User(error_user.id.clone()));

    let error = || CommandError::from_ctx(&ctx);

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

    if other_user.id == ctx.author().id {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("You can't challenge yourself!"),
        )
        .change_context_lazy(error)
        .attach(error_user)
        .await?;

        return Ok(());
    }

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
        .attach_lazy(&error_user)
        .await
    {
        Ok(Some(opponent)) => start_game_both_discord(ctx, side, author, opponent).await,
        Ok(None) => {
            message
                .edit(
                    &ctx.http(),
                    EditMessage::default()
                        .content(
                            "The other user has declined the challenge. Better luck next time!",
                        )
                        .components(vec![]),
                )
                .change_context_lazy(error)
                .attach_lazy(&error_user)
                .await
        }
        Err(err) => match err.current_context() {
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
        },
    }
}

async fn start_game_both_discord(
    ctx: Context<'_>,
    side: GameChoice,
    author: PlayerPlatform,
    opponent: PlayerPlatform,
) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&ctx);

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
            let PlayerPlatform::Discord {
                mut game_message, ..
            } = author
            else {
                unreachable!()
            };

            game_message
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
    match backend.create_game(white.clone(), black.clone()).await {
        Ok(()) => Ok(()),
        Err(err) => match err.current_context() {
            CreateGameError::PlayerInGame => {
                let PlayerPlatform::Discord {
                    game_message: mut white_message,
                    ..
                } = white
                else {
                    unreachable!()
                };

                let PlayerPlatform::Discord {
                    game_message: mut black_message,
                    ..
                } = black
                else {
                    unreachable!()
                };

                white_message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content("One of y'all are already in a game!")
                            .components(vec![]),
                    )
                    .change_context_lazy(error)
                    .await?;

                black_message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content("One of y'all are already in a game!")
                            .components(vec![]),
                    )
                    .change_context_lazy(error)
                    .await
            }
            CreateGameError::UsernameTaken(user) => {
                let PlayerPlatform::Discord {
                    user,
                    mut game_message,
                    ..
                } = (match user {
                    Color::White => white,
                    Color::Black => black,
                })
                else {
                    unreachable!()
                };

                game_message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content(format!(
                                "{}, your discord username ({}) is already taken! Use `/create_account` to manually create your account and choose a username.",
                                user.mention(),
                                user.name
                            ))
                            .components(vec![]),
                    )
                    .change_context_lazy(error)
                    .await
            }
            _ => return Err(err.change_context(error())),
        },
    }
}
