use backend::{BackendService, Service};
use discord::DiscordBotService;
use error_stack::{bail, FutureExt, Result, ResultExt};
use std::{fmt, sync::Arc};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;

mod env;

#[derive(Debug)]
enum MainError {
    EnvError,
    LogError,
    ServiceError,
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EnvError => f.write_str("Environment variable error"),
            Self::LogError => f.write_str("Logging error"),
            Self::ServiceError => f.write_str("Service error"),
        }
    }
}

impl std::error::Error for MainError {}

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<(), MainError> {
    // init environment variables
    dotenvy::dotenv().change_context(MainError::EnvError)?;

    if env::any_set() {
        env::assert_env_vars();
    } else {
        error!(
            "# Environment Variables Failed to load. Errors:\n{}",
            env::gen_help()
        );
        bail!(MainError::EnvError)
    }

    info!("Environment variables loaded successfully");

    // init logging/tracing
    let console_layer = tracing_subscriber::fmt::layer().pretty();

    let subscriber = tracing_subscriber::registry().with(console_layer).with(
        tracing_subscriber::EnvFilter::builder()
            .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
            .from_env()
            .change_context(MainError::LogError)?,
    );

    tracing::subscriber::set_global_default(subscriber).change_context(MainError::LogError)?;

    // initialize services

    let backend_service = Arc::new(
        BackendService::new(env::database_url())
            .await
            .attach_printable("Failed to initialize backend service")
            .change_context(MainError::ServiceError)?,
    );

    backend_service
        .run()
        .await
        .attach_printable("Failed to run backend service")
        .change_context(MainError::ServiceError)?;

    let discord_bot_task = tokio::task::spawn(
        DiscordBotService::new(env::discord_token(), backend_service.clone())
            .run()
            .attach_printable("Failed to run discord bot service"),
    );

    info!("Services initialized successfully");

    // wait for services to finish
    discord_bot_task
        .await
        .attach_printable("Failed to wait for discord bot service")
        .change_context(MainError::ServiceError)?
        .attach_printable("Failed to run discord bot service")
        .change_context(MainError::ServiceError)?;

    Ok(())
}
