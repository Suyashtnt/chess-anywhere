use chess::{ChessError, ChessGame, SanArray};
use dashmap::{mapref::multiple::RefMutMulti, DashMap};
use error_stack::{bail, ensure, FutureExt, Result, ResultExt};
use poise::serenity_prelude::{
    futures::{future::OptionFuture, TryFutureExt},
    model::user,
    UserId,
};
use shakmaty::{san::San, Color, Move, Outcome};
use skillratings::Outcomes;
use std::{error::Error, fmt, future::Future, sync::Arc};
use users::{Player, PlayerPlatform, UpdateBoardError};

pub mod chess;
pub mod users;

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
    pool: sqlx::postgres::PgPool,
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

type Game<'a> = RefMutMulti<'a, (Player, Player), ChessGame>;

impl BackendService {
    pub async fn new(db_url: String) -> Result<Self, sqlx::Error> {
        let pg_pool = sqlx::postgres::PgPool::connect(&db_url).await?;

        Ok(Self {
            pool: pg_pool,
            current_games: Arc::new(DashMap::new()),
        })
    }

    pub async fn create_game(
        &self,
        white: PlayerPlatform,
        black: PlayerPlatform,
    ) -> Result<(), CreateGameError> {
        let white = Player::upsert(white, &self.pool)
            .change_context(CreateGameError::DatabaseError)
            .await?;

        let black = Player::upsert(black, &self.pool)
            .change_context(CreateGameError::DatabaseError)
            .await?;

        let mut user_tuple = (white, black);

        if self.current_games.contains_key(&user_tuple) {
            bail!(CreateGameError::PlayerInGame);
        }

        for entry in self.current_games.iter() {
            let (white, black) = entry.key();

            ensure!(
                white.id() != user_tuple.0.id()
                    || black.id() != user_tuple.0.id()
                    || white.id() != user_tuple.1.id()
                    || black.id() != user_tuple.1.id(),
                CreateGameError::PlayerInGame
            );
        }

        let game = ChessGame::new();
        let board = game.board();

        Self::update_board(
            &mut user_tuple.0,
            &mut user_tuple.1,
            Color::White,
            None,
            board,
        )
        .change_context(CreateGameError::DatabaseError)
        .await?;

        self.current_games.insert(user_tuple, game);

        Ok(())
    }

    async fn update_board(
        white: &mut Player,
        black: &mut Player,
        current_player: Color,
        last_move: Option<Move>,
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
                white
                    .update_board(
                        board,
                        current_player,
                        last_move,
                        black.username(),
                        current_player == Color::White,
                    )
                    .await
            }
            _ => {
                white
                    .update_board(
                        board,
                        current_player,
                        last_move.clone(),
                        black.username(),
                        current_player == Color::White,
                    )
                    .await?;

                black
                    .update_board(
                        board,
                        current_player,
                        last_move,
                        white.username(),
                        current_player == Color::Black,
                    )
                    .await
            }
        }
    }

    /// Plays a move for the given player
    ///
    /// # Returns
    /// Returns Ok(true) if the game is over,
    /// Ok(false) if the game is not over,
    /// and Err(ChessError) if the move is invalid or it is not the player's turn
    ///
    /// Handling stopping the game for the players is up to the caller.
    /// We automatically remove the running game and update ELOs
    pub async fn play_move(
        &mut self,
        player: PlayerPlatform,
        san: San,
    ) -> Result<Option<Outcome>, ChessError> {
        let Some(mut game) = self.get_game(&player) else {
            bail!(ChessError::InvalidPlayer)
        };

        let current_player_color = Color::from_white(player == *game.key().0.platform());

        if let Some(outcome) = game.play_move(&current_player_color, san)? {
            let key = game.key().to_owned();
            drop(game);
            self.handle_game_over(&key, outcome).await?;
            Ok(Some(outcome))
        } else {
            Ok(None)
        }
    }

    /// Gets the game for a specific player on discord
    pub async fn find_player_discord(&self, id: UserId) -> Option<PlayerPlatform> {
        self.current_games.iter().find_map(|entry| {
            let (white, black) = entry.key();
            if let PlayerPlatform::Discord { user, .. } = white.platform() {
                if user.id == id {
                    return Some(white.platform().clone());
                }
            }

            if let PlayerPlatform::Discord { user, .. } = black.platform() {
                if user.id == id {
                    return Some(black.platform().clone());
                }
            }

            None
        })
    }

    /// Gets the valid moves for the given player
    pub async fn get_moves(&self, player: PlayerPlatform) -> SanArray {
        todo!()
    }

    async fn handle_game_over<'a>(
        &'a mut self,
        key: &(Player, Player),
        outcome: Outcome,
    ) -> Result<(), ChessError> {
        // we know the game exists, so unwrap is safe
        let ((mut white, mut black), _game) = self.current_games.remove(key).unwrap();
        if white == black {
            return Ok(());
        }

        let elo_outcome = match outcome {
            Outcome::Draw => Outcomes::DRAW,
            Outcome::Decisive {
                winner: Color::White,
            } => Outcomes::WIN,
            Outcome::Decisive {
                winner: Color::Black,
            } => Outcomes::LOSS,
        };

        white
            .update_elo(&mut black, elo_outcome, &self.pool)
            .change_context(ChessError::DatabaseError)
            .await?;

        Ok(())
    }

    /// Gets the game for a specific player on a specific platform
    ///
    /// # Returns
    /// Returns None if the player is not in a game,
    /// else returns white, black, and the game
    fn get_game<'a>(&'a self, player: &PlayerPlatform) -> Option<Game<'a>> {
        self.current_games.iter_mut().find(|entry| {
            let (white, black) = entry.key();
            white.platform() == player || black.platform() == player
        })
    }

    /// Initializes the database and fills up various caches
    #[tracing::instrument]
    pub async fn run(&self) -> Result<(), ServiceError> {
        sqlx::migrate!()
            .run(&self.pool)
            .await
            .change_context(ServiceError)?;

        Ok(())
    }
}
