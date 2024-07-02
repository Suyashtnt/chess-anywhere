pub mod board;
pub mod error;
mod move_piece;
mod new_game;

use core::fmt;
use std::{sync::Arc, time::Duration};

use crate::{
    backend::{players::PlayerPlatform, CreateGameError, ServiceError},
    BACKEND_SERVICE,
};
use error::{Argument, CommandError};
use error_stack::{bail, FutureExt, Report, Result, ResultExt};
use poise::{
    serenity_prelude::{
        self as serenity, ButtonStyle, ComponentInteractionCollector, CreateActionRow,
        CreateButton, CreateEmbed, CreateMessage, EditMessage, Mentionable, ShardId, UserId,
    },
    CreateReply,
};
use tokio::task::JoinHandle;
use tracing::error;

pub struct DiscordBotService {
    http: Arc<serenity::http::Http>,
    shard_manager: Arc<serenity::ShardManager>,
}

impl fmt::Debug for DiscordBotService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiscordBotService").finish()
    }
}

impl DiscordBotService {
    pub async fn start(
        token: String,
    ) -> Result<(Self, JoinHandle<Result<(), ServiceError>>), ServiceError> {
        let intents = serenity::GatewayIntents::non_privileged();

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![move_piece::r#move(), new_game::new_game(), register()],
                on_error: |error| {
                    Box::pin(async move {
                        match error {
                            poise::FrameworkError::Command { error, ctx, .. } => {
                                let mut error_embed = CreateEmbed::new()
                                    .title("Error")
                                    .description(error.to_string());

                                error!("{} failed: {:?}", ctx.command().name, error);

                                for argument in error.request_ref::<Argument>() {
                                    error_embed = error_embed.field(
                                        argument.0.clone(),
                                        format!("{}", argument.1),
                                        false,
                                    );
                                }

                                ctx.send(CreateReply::default().embed(error_embed))
                                    .await
                                    .unwrap();
                            }
                            other => poise::builtins::on_error(other).await.unwrap(),
                        }
                    })
                },
                ..Default::default()
            })
            .setup(|_ctx, _ready, _framework| Box::pin(async move { Ok(()) }))
            .build();

        let mut client = serenity::ClientBuilder::new(&token, intents)
            .framework(framework)
            .await
            .change_context(ServiceError)?;

        let http = client.http.clone();
        let shard_manager = client.shard_manager.clone();

        let future =
            tokio::task::spawn(async move { client.start().change_context(ServiceError).await });

        Ok((
            Self {
                http,
                shard_manager,
            },
            future,
        ))
    }

    /// Sends a challenge to the user in a DM
    ///
    /// # Returns
    /// Returns Ok(Some(message)) if the user accepted the challenge (and the message is the game board message),
    ///
    /// Ok(None) if the user declined the challenge or did not respond in time,
    ///
    /// and Err(ChallengeError) if the challenge could not be sent
    pub async fn challenge_user_discord(
        &self,
        your_username: &str,
        user_id: UserId,
    ) -> Result<Option<PlayerPlatform>, CreateGameError> {
        let backend = BACKEND_SERVICE.get().unwrap();

        if backend.find_player_discord(user_id).await.is_some() {
            bail!(CreateGameError::PlayerInGame);
        }

        let channel = user_id
            .create_dm_channel(&self.http)
            .change_context(CreateGameError::DiscordError)
            .await?;

        let components = vec![CreateActionRow::Buttons(vec![
            CreateButton::new("accept")
                .label("Accept")
                .style(ButtonStyle::Primary),
            CreateButton::new("decline")
                .label("Decline")
                .style(ButtonStyle::Danger),
        ])];

        let mut message = channel
            .send_message(
                &self.http,
                CreateMessage::new()
                    .content(format!(
                        "{}, you have been challenged to a game of chess by {}! Do you accept?",
                        user_id.mention(),
                        your_username
                    ))
                    .components(components),
            )
            .change_context(CreateGameError::DiscordError)
            .await?;

        let shard_runner = self
            .shard_manager
            .runners
            .lock()
            .await
            .get(&ShardId(0))
            .unwrap()
            .runner_tx
            .clone();

        let response = ComponentInteractionCollector::new(&shard_runner)
            .author_id(user_id)
            .message_id(message.id)
            .timeout(Duration::from_secs(60))
            .await;

        match response {
            Some(response) => match response.data.custom_id == "accept" {
                true => {
                    message
                        .edit(
                            &self.http,
                            EditMessage::default()
                                .content(
                                    "You have accepted the challenge! The game will begin shortly.",
                                )
                                .components(vec![]),
                        )
                        .change_context(CreateGameError::DiscordError)
                        .await?;

                    Ok(Some(PlayerPlatform::Discord {
                        user: user_id
                            .to_user(&self.http)
                            .change_context(CreateGameError::DiscordError)
                            .await?,
                        game_message: message,
                        http: self.http.clone(),
                    }))
                }
                false => {
                    message
                        .edit(
                            &self.http,
                            EditMessage::default()
                                .content(format!("Declined! {} will be notified.", your_username))
                                .components(vec![]),
                        )
                        .change_context(CreateGameError::DiscordError)
                        .await?;

                    Ok(None)
                }
            },
            None => {
                message
                    .reply(
                        &self.http,
                        format!(
                            "You took too long to respond. {} will be notified",
                            your_username
                        ),
                    )
                    .change_context(CreateGameError::DiscordError)
                    .await?;

                Ok(None)
            }
        }
    }
}

pub(crate) type Error = Report<CommandError>;
pub(crate) type Context<'a> = poise::Context<'a, (), Error>;

#[poise::command(prefix_command, slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), CommandError> {
    poise::builtins::register_application_commands_buttons(ctx)
        .await
        .attach_printable("Could not do registration")
        .change_context(CommandError::from_ctx(&ctx))?;

    Ok(())
}
