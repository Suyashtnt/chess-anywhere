use error_stack::{FutureExt, Result};
use poise::{
    serenity_prelude::{CreateMessage, EditMessage},
    Modal,
};

use crate::{
    discord::{error::CommandError, Context},
    users::UserService,
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
/// Link an email to your Discord-made account (to link a pre-existing account, use the website)
pub async fn email(ctx: ApplicationContext<'_>) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&poise::Context::Application(ctx));

    let dm_channel = ctx
        .author()
        .create_dm_channel(ctx.http())
        .change_context_lazy(&error)
        .await?;

    let Some(data) = EmailModal::execute(ctx.clone())
        .change_context_lazy(&error)
        .await?
    else {
        return dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default().content("You didn't respond to the modal!"),
            )
            .change_context_lazy(error)
            .await
            .map(|_| ());
    };

    let Some(user) = UserService::fetch_user_by_discord_id(ctx.author().id, &ctx.data.pool)
        .change_context_lazy(&error)
        .await?
    else {
        return dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default()
                    .content("You don't have an account! Use `/create_account` to create one."),
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

    api_service
        .state
        .send_magic_email(&data.email, user.id())
        .change_context_lazy(&error)
        .await?;

    message
        .edit(ctx.http(), EditMessage::default().content("Email sent!"))
        .change_context_lazy(&error)
        .await?;

    Ok(())
}

#[derive(Debug, poise::Modal)]
struct UsernameModal {
    /// The username you wish to use
    username: String,
}

#[poise::command(slash_command)]
#[tracing::instrument]
/// Manually create a chess-anywhere account
pub async fn create_account(ctx: ApplicationContext<'_>) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&poise::Context::Application(ctx));

    let dm_channel = ctx
        .author()
        .create_dm_channel(ctx.http())
        .change_context_lazy(&error)
        .await?;

    let Some(data) = UsernameModal::execute(ctx.clone())
        .change_context_lazy(&error)
        .await?
    else {
        return dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default().content("You didn't respond to the modal!"),
            )
            .change_context_lazy(error)
            .await
            .map(|_| ());
    };

    if UserService::fetch_user_by_username(&data.username, &ctx.data.pool)
        .await
        .is_ok_and(|user| user.is_some())
    {
        dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default()
                    .content("Username already taken! Try again with a different username."),
            )
            .change_context_lazy(&error)
            .await?;
    }

    UserService::create(&data.username, &ctx.data.pool)
        .change_context_lazy(&error)
        .await?;

    dm_channel
        .send_message(
            ctx.http(),
            CreateMessage::default().content("Account created!"),
        )
        .change_context_lazy(&error)
        .await?;

    Ok(())
}

#[poise::command(slash_command)]
#[tracing::instrument]
/// Change your username on chess-anywhere
pub async fn change_username(ctx: ApplicationContext<'_>) -> Result<(), CommandError> {
    let error = || CommandError::from_ctx(&poise::Context::Application(ctx));

    let dm_channel = ctx
        .author()
        .create_dm_channel(ctx.http())
        .change_context_lazy(&error)
        .await?;

    let Some(mut user) = UserService::fetch_user_by_discord_id(ctx.author().id, &ctx.data.pool)
        .change_context_lazy(&error)
        .await?
    else {
        return dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default()
                    .content("You don't have an account! Use `/create_account` to create one."),
            )
            .change_context_lazy(error)
            .await
            .map(|_| ());
    };

    let Some(data) = UsernameModal::execute(ctx.clone())
        .change_context_lazy(&error)
        .await?
    else {
        return dm_channel
            .send_message(
                ctx.http(),
                CreateMessage::default().content("You didn't respond to the modal!"),
            )
            .change_context_lazy(error)
            .await
            .map(|_| ());
    };

    user.update_username(data.username, &ctx.data.pool)
        .change_context_lazy(&error)
        .await?;

    dm_channel
        .send_message(
            ctx.http(),
            CreateMessage::default().content("Username changed!"),
        )
        .change_context_lazy(&error)
        .await?;

    Ok(())
}
