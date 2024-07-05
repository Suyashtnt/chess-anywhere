mod api;
mod backend;
mod discord;
mod users;

use api::ApiService;
use backend::BackendService;
use discord::DiscordBotService;
use error_stack::{bail, FutureExt, Result, ResultExt};
use sqlx::SqlitePool;
use std::fmt;
use tokio::{select, sync::OnceCell};
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;

mod env;

pub static BACKEND_SERVICE: OnceCell<BackendService> = OnceCell::const_new();
pub static DISCORD_BOT_SERVICE: OnceCell<DiscordBotService> = OnceCell::const_new();
pub static API_SERVICE: OnceCell<ApiService> = OnceCell::const_new();

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
    let pool = SqlitePool::connect(&env::database_url())
        .await
        .change_context(MainError::ServiceError)?;

    BACKEND_SERVICE
        .set(
            BackendService::new(pool.clone())
                .await
                .attach_printable("Failed to initialize backend service")
                .change_context(MainError::ServiceError)?,
        )
        .unwrap();

    let (bot_service, bot_task) = DiscordBotService::start(env::discord_token(), pool.clone())
        .change_context(MainError::ServiceError)
        .await?;

    DISCORD_BOT_SERVICE.set(bot_service).unwrap();

    let (api_service, api_task) = ApiService::start(
        &env::resend_api_key(),
        pool.clone(),
        async {
            select! {
                _ = bot_task => (),
                _ = tokio::signal::ctrl_c() => (),
            }
        },
        3000,
    )
    .change_context(MainError::ServiceError)
    .await?;

    API_SERVICE.set(api_service).unwrap();

    info!("Services initialized successfully");

    api_task
        .change_context(MainError::ServiceError)
        .await?
        .change_context(MainError::ServiceError)?;

    Ok(())
}
