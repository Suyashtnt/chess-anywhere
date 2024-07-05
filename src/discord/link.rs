use error_stack::{FutureExt, Result};
use poise::{
    serenity_prelude::{CreateMessage, EditMessage},
    Modal,
};

use crate::{
    backend::players::Player,
    discord::{error::CommandError, Context},
    API_SERVICE,
};

use super::ApplicationContext;

#[poise::command(slash_command, subcommands("email"), subcommand_required)]
pub async fn link(_: Context<'_>) -> Result<(), CommandError> {
    Ok(())
}

#[derive(Debug, poise::Modal)]
struct EmailModal {
    /// The email you wish to link
    email: String,
}

#[poise::command(slash_command)]
#[tracing::instrument]
/// Link your chess-anywhere account created on Discord with your email (a modal will pop up)
pub async fn email(ctx: ApplicationContext<'_>) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&poise::Context::Application(ctx));

    let Some(data) = EmailModal::execute(ctx.clone())
        .change_context_lazy(&error)
        .await?
    else {
        return ctx
            .author()
            .create_dm_channel(ctx.http())
            .change_context_lazy(&error)
            .await?
            .send_message(
                ctx.http(),
                CreateMessage::default().content("You didn't respond to the modal!"),
            )
            .change_context_lazy(error)
            .await
            .map(|_| ());
    };

    let mut message = ctx
        .author()
        .create_dm_channel(ctx.http())
        .change_context_lazy(&error)
        .await?
        .send_message(
            ctx.http(),
            CreateMessage::default().content("Sending email..."),
        )
        .change_context_lazy(&error)
        .await?;

    let api_service = API_SERVICE.get().unwrap();

    let player = Player::upsert(
        crate::backend::players::PlayerPlatform::Discord {
            user: ctx.author().clone(),
            game_message: message.clone(),
            http: ctx.serenity_context().http.clone(),
        },
        &ctx.data.pool,
    )
    .change_context_lazy(&error)
    .await?;

    api_service
        .state
        .send_magic_email(&data.email, player.id())
        .change_context_lazy(&error)
        .await?;

    message
        .edit(ctx.http(), EditMessage::default().content("Email sent!"))
        .change_context_lazy(&error)
        .await?;

    Ok(())
}
