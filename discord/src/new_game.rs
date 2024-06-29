use std::time::Duration;

use crate::{
    error::{Arg, CommandError},
    Context,
};
use backend::{users::PlayerPlatform, CreateGameError};
use error_stack::{Result, ResultExt};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, EditMessage,
        Mentionable, Message, User,
    },
    CreateReply,
};

#[poise::command(slash_command, subcommands("discord"), subcommand_required)]
pub async fn new_game(_: Context<'_>) -> Result<(), CommandError> {
    Ok(())
}

#[poise::command(slash_command)]
#[tracing::instrument]
pub async fn discord(
    ctx: Context<'_>,
    #[description = "The user you want to play against"] other_user: User,
) -> Result<(), CommandError> {
    let error = || {
        let error_user = other_user.clone();
        CommandError::from_ctx(&ctx, vec![Arg::User(error_user.name, error_user.id)])
    };

    if other_user.bot {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("You can't challenge a bot!"),
        )
        .await
        .change_context_lazy(error)?;

        return Ok(());
    }

    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{}-accept", other_user.id))
            .label("Accept")
            .style(ButtonStyle::Primary),
        CreateButton::new(format!("{}-decline", other_user.id))
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
        .change_context_lazy(error)?;

    let mut message = challenge_message
        .into_message()
        .await
        .change_context_lazy(error)?;

    let response = ComponentInteractionCollector::new(ctx)
        .author_id(other_user.id)
        .message_id(message.id)
        .timeout(Duration::from_secs(60))
        .filter(move |mci| mci.data.custom_id.starts_with(&other_user.id.to_string()))
        .await;

    match response {
        Some(response) => match response.data.custom_id.ends_with("accept") {
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
                    .await
                    .change_context_lazy(error)?;

                start_game_both_discord(ctx, other_user, message).await?;
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
                    .await
                    .change_context_lazy(error)?;
            }
        },
        None => {
            message
                .reply(
                    &ctx.http(),
                    format!("@silent {} took too long to respond", other_user.mention()),
                )
                .await
                .change_context_lazy(error)?;
        }
    };

    Ok(())
}

async fn start_game_both_discord(
    ctx: Context<'_>,
    other_user: User,
    mut message: Message,
) -> Result<(), CommandError> {
    let error_user = other_user.clone();
    let error = || CommandError::from_ctx(&ctx, vec![Arg::User(error_user.name, error_user.id)]);

    let white = PlayerPlatform::Discord {
        user: ctx.author().clone(),
        game_message: message.clone(),
        context: ctx.serenity_context().clone(),
    };

    let black = PlayerPlatform::Discord {
        user: other_user,
        game_message: message.clone(),
        context: ctx.serenity_context().clone(),
    };

    let res = ctx.data().backend.create_game(white, black).await;

    match res {
        Ok(()) => Ok(()),
        Err(err) => {
            let current_frame: &CreateGameError =
                err.frames().next().unwrap().downcast_ref().unwrap();

            match current_frame {
                CreateGameError::PlayerInGame => message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content("You are already in a game!")
                            .components(vec![]),
                    )
                    .await
                    .change_context_lazy(error),
                _ => return Err(err.change_context(error())),
            }
        }
    }
}
