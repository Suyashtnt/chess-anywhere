pub mod board;
pub mod error;
mod move_piece;
mod new_game;

use crate::backend::ServiceError;
use error::{Argument, CommandError};
use error_stack::{Report, Result, ResultExt};
use poise::{
    serenity_prelude::{self as serenity, CreateEmbed},
    CreateReply,
};
use tracing::error;

#[derive(Debug)]
pub struct DiscordBotService {
    token: String,
}

impl DiscordBotService {
    pub fn new(token: String) -> Self {
        Self { token }
    }

    #[tracing::instrument]
    pub async fn run(&self) -> Result<(), ServiceError> {
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
            .build();

        let mut client = serenity::ClientBuilder::new(&self.token, intents)
            .framework(framework)
            .await
            .change_context(ServiceError)?;

        client.start().await.change_context(ServiceError)
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
