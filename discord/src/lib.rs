mod error;
mod move_piece;
mod new_game;

use std::sync::Arc;

use backend::{BackendService, Service, ServiceError};
use error::CommandError;
use error_stack::{Report, Result, ResultExt};
use poise::serenity_prelude as serenity;

#[derive(Debug)]
pub struct DiscordBotService {
    token: String,
    backend: Arc<BackendService>,
}

impl DiscordBotService {
    pub fn new(token: String, backend: Arc<BackendService>) -> Self {
        Self { token, backend }
    }
}

pub struct Data {
    pub(crate) backend: Arc<BackendService>,
}
pub type Error = Report<CommandError>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), CommandError> {
    poise::builtins::register_application_commands_buttons(ctx)
        .await
        .attach_printable("Could not do registration")
        .change_context(CommandError::from_ctx(&ctx, vec![]))?;

    Ok(())
}

impl Service for DiscordBotService {
    const SERVICE_NAME: &'static str = "Discord Bot";

    #[tracing::instrument]
    async fn run(self) -> Result<(), backend::ServiceError> {
        let intents = serenity::GatewayIntents::non_privileged();

        let framework_backend = self.backend.clone();
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![
                    move_piece::r#move(),
                    new_game::new_game(),
                    new_game::new_external_game(),
                ],
                ..Default::default()
            })
            .setup(|ctx, _ready, framework| {
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands)
                        .await
                        .unwrap();

                    Ok(Data {
                        backend: framework_backend,
                    })
                })
            })
            .build();

        let mut client = serenity::ClientBuilder::new(&self.token, intents)
            .framework(framework)
            .await
            .change_context(ServiceError)?;

        client.start().await.change_context(ServiceError)
    }
}
