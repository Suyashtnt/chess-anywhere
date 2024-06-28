use auth::{Player, PlayerPlatform, UpdateBoardError};
use chess::{ChessError, ChessGame};
use dashmap::DashMap;
use error_stack::{bail, FutureExt, Result, ResultExt};
use poise::serenity_prelude::futures::{future::OptionFuture, TryFutureExt};
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

#[derive(Debug)]
pub enum CreateGameError {
    PlayerInGame,
    PlayerDoesNotExist,
    DatabaseError,
}

impl fmt::Display for CreateGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PlayerInGame => f.write_str("One of the players is already in a game!"),
            Self::PlayerDoesNotExist => f.write_str("One of the players does not exist!"),
            Self::DatabaseError => f.write_str("Database error!"),
        }
    }
}

impl std::error::Error for CreateGameError {}

impl BackendService {
    pub async fn new(db_url: String) -> Result<Self, sqlx::Error> {
        let pg_pool = sqlx::postgres::PgPool::connect(&db_url).await?;

        Ok(Self {
            pg_pool,
            current_games: Arc::new(DashMap::new()),
        })
    }

    pub async fn create_game(
        &self,
        white: PlayerPlatform,
        black: PlayerPlatform,
    ) -> Result<(), CreateGameError> {
        let white = Player::upsert(white, &self.pg_pool, || todo!())
            .change_context(CreateGameError::DatabaseError)
            .await?;

        let black = Player::upsert(black, &self.pg_pool, || todo!())
            .change_context(CreateGameError::DatabaseError)
            .await?;

        let mut user_tuple = (white, black);

        if self.current_games.contains_key(&user_tuple) {
            bail!(CreateGameError::PlayerInGame);
        }

        let game = ChessGame::new();
        let board = game.board();

        Self::update_board(&mut user_tuple.0, &mut user_tuple.1, board)
            .change_context(CreateGameError::DatabaseError)
            .await?;

        self.current_games.insert(user_tuple, game);

        Ok(())
    }

    async fn update_board(
        white: &mut Player,
        black: &mut Player,
        board: &shakmaty::Board,
    ) -> Result<(), UpdateBoardError> {
        match (white.platform(), black.platform()) {
            (
                PlayerPlatform::Discord {
                    game_message: white_msg,
                    ..
                },
                PlayerPlatform::Discord {
                    game_message: black_msg,
                    ..
                },
            ) if white_msg.id == black_msg.id => {
                // just update the board for the white player, and black will automatically get updated since it's the same message
                white.update_board(board).await
            }
            _ => {
                white.update_board(board).await?;
                black.update_board(board).await
            }
        }
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
