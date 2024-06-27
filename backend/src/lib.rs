use auth::{Player, PlayerPlatform};
use chess::{ChessError, ChessGame};
use dashmap::DashMap;
use error_stack::{Result, ResultExt};
use shakmaty::san::San;
use std::{error::Error, fmt, future::Future, sync::Arc};

pub mod auth;
pub mod chess;

#[derive(Debug)]
pub struct ServiceError;

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Service error")
    }
}

impl Error for ServiceError {}

pub trait Service {
    /// The name of the service used in logs
    const SERVICE_NAME: &'static str;

    /// Run the service
    ///
    /// # Returns
    /// Returns OK(()) If it runs successfully and it was intended to end
    /// Else returns an error if it fails (and consequently takes down the whole app)
    fn run(self) -> impl Future<Output = Result<(), ServiceError>> + Send;
}

#[derive(Debug, Clone)]
pub struct BackendService {
    pg_pool: sqlx::postgres::PgPool,
    current_games: Arc<DashMap<(Player, Player), ChessGame>>,
}

pub enum CreateGameError {
    PlayerInGame,
    PlayerDoesNotExist,
}

impl BackendService {
    pub async fn new(db_url: String) -> Result<Self, sqlx::Error> {
        let pg_pool = sqlx::postgres::PgPool::connect(&db_url).await?;

        Ok(Self {
            pg_pool,
            current_games: Arc::new(DashMap::new()),
        })
    }

    pub fn create_game(
        &self,
        white: PlayerPlatform,
        black: PlayerPlatform,
    ) -> Result<(), CreateGameError> {
        todo!()
    }

    pub fn play_move(&self, player: PlayerPlatform, san: San) -> Result<(), ChessError> {
        todo!()
    }

    fn handle_game_over(&self, game: ChessGame) {
        todo!()
    }

    /// Initializes the database and fills up various caches
    #[tracing::instrument]
    pub async fn run(&self) -> Result<(), ServiceError> {
        sqlx::migrate!()
            .run(&self.pg_pool)
            .await
            .change_context(ServiceError)?;

        Ok(())
    }
}
