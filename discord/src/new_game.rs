use std::time::Duration;

use crate::{
    error::{Arg, CommandError},
    Context,
};
use error_stack::{Result, ResultExt};
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, EditMessage,
        Mentionable, User,
    },
    CreateReply,
};
use tracing::info;

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

    info!("Waiting for response from {}", other_user.name);

    let response = ComponentInteractionCollector::new(ctx)
        .message_id(message.id)
        .timeout(Duration::from_secs(60))
        .filter(move |mci| mci.data.custom_id.starts_with(&other_user.id.to_string()))
        .await;

    match response {
        Some(response) => match response.data.custom_id.ends_with("accept") {
            true => {
                message
                    .edit(
                        &ctx.http(),
                        EditMessage::default()
                            .content(format!(
                                "{} has accepted the challenge! The game will begin shortly.",
                                other_user.mention()
                            ))
                            .components(vec![]),
                    )
                    .await
                    .change_context_lazy(error)?;

                start_game_both_discord(ctx, other_user).await?;
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

async fn start_game_both_discord(ctx: Context<'_>, other_user: User) -> Result<(), CommandError> {
    let error = || {
        let error_user = other_user.clone();
        CommandError::from_ctx(&ctx, vec![Arg::User(error_user.name, error_user.id)])
    };

    // TODOs: add game to backend and prepare listeners for moves

    todo!();

    Ok(())
}
