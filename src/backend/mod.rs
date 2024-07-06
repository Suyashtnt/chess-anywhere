use chess::{ChessError, ChessGame, MoveStatus, SanArray};
use dashmap::{mapref::multiple::RefMutMulti, DashMap};
use error_stack::{bail, ensure, FutureExt, Result, ResultExt};
use players::{Player, PlayerPlatform, UpdateBoardError};
use poise::serenity_prelude::UserId;
use shakmaty::{san::San, Board, Chess, Color, Move, Outcome};
use skillratings::Outcomes;
use std::{
    error::Error,
    fmt::{self, Debug},
    sync::Arc,
};

use crate::users::GameOutcome;

pub mod chess;
pub mod players;

pub const DRAW_OFFER_SAN: &str = "=";

#[derive(Debug)]
pub struct ChallengeError;
impl std::fmt::Display for ChallengeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Failed to send challenge")
    }
}
impl std::error::Error for ChallengeError {}

#[derive(Debug)]
pub struct ServiceError;

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Service error")
    }
}

impl Error for ServiceError {}

#[derive(Debug, Clone)]
pub struct BackendService {
    pool: sqlx::SqlitePool,
    current_games: Arc<DashMap<(Player, Player), ChessGame>>,
}

#[derive(Debug)]
pub enum CreateGameError {
    PlayerInGame,
    PlayerDoesNotExist,
    UsernameTaken(Color),
    DatabaseError,
    DiscordError,
}

impl fmt::Display for CreateGameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PlayerInGame => f.write_str("One of the players is already in a game!"),
            Self::PlayerDoesNotExist => f.write_str("One of the players does not exist!"),
            Self::DatabaseError => f.write_str("Database error!"),
            Self::DiscordError => f.write_str("Discord error!"),
            Self::UsernameTaken(color) => {
                write!(
                    f,
                    "{}'s Username is already taken! You'll need to manually create an account.",
                    color
                )
            }
        }
    }
}

impl std::error::Error for CreateGameError {}

type Game<'a> = RefMutMulti<'a, (Player, Player), ChessGame>;

#[derive(Debug)]
pub(crate) struct GameInfo<'a> {
    pub id: i64,
    pub white: &'a mut Player,
    pub black: &'a mut Player,
    pub last_move: &'a MoveStatus,
    pub current_player: Color,
    pub board: &'a Board,
    pub position: &'a Chess,
}

impl BackendService {
    pub async fn new(pool: sqlx::SqlitePool) -> Result<Self, ServiceError> {
        sqlx::migrate!()
            .run(&pool)
            .await
            .change_context(ServiceError)?;

        Ok(Self {
            pool,
            current_games: Arc::new(DashMap::new()),
        })
    }

    pub async fn create_game(
        &self,
        white: PlayerPlatform,
        black: PlayerPlatform,
    ) -> Result<(), CreateGameError> {
        let Some(white) = Player::upsert(white, &self.pool)
            .change_context(CreateGameError::DatabaseError)
            .await?
        else {
            bail!(CreateGameError::UsernameTaken(Color::White))
        };

        let Some(black) = Player::upsert(black, &self.pool)
            .change_context(CreateGameError::DatabaseError)
            .await?
        else {
            bail!(CreateGameError::UsernameTaken(Color::Black))
        };

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

        let white_id = user_tuple.0.id();
        let black_id = user_tuple.1.id();

        let game_id = sqlx::query!(
            "
            INSERT INTO games (white_id, black_id) VALUES (?, ?)
            RETURNING id
            ",
            white_id,
            black_id
        )
        .fetch_one(&self.pool)
        .await
        .change_context(CreateGameError::DatabaseError)?
        .id;

        let game = ChessGame::new(game_id);

        let mut game_info = GameInfo {
            id: game_id,
            white: &mut user_tuple.0,
            black: &mut user_tuple.1,
            last_move: &MoveStatus::GameStart,
            current_player: Color::White,
            board: game.board(),
            position: game.position(),
        };

        self.update_board(&mut game_info)
            .change_context(CreateGameError::DatabaseError)
            .await?;

        self.current_games.insert(user_tuple, game);

        Ok(())
    }

    async fn add_move_to_db(
        &self,
        info: &mut GameInfo<'_>,
        game_move: &Move,
    ) -> Result<(), UpdateBoardError> {
        let last_player_id = if info.current_player == Color::White {
            info.white.id()
        } else {
            info.black.id()
        };

        let san = San::from_move(info.position, game_move).to_string();

        sqlx::query!(
            "
                    INSERT INTO moves (player_id, game_id, move, move_number)
                    VALUES (
                        $1,
                        $2,
                        $3,
                        (
                            SELECT (move_number + 1) FROM moves WHERE game_id = $2
                            UNION
                            SELECT 1 as move_number
                            ORDER BY move_number DESC LIMIT 1
                        )
                    )
                ",
            last_player_id,
            info.id,
            san
        )
        .execute(&self.pool)
        .change_context(UpdateBoardError::DatabaseError)
        .await?;

        Ok(())
    }

    async fn set_game_outcome(
        &self,
        game_info: &mut GameInfo<'_>,
        outcome: Outcome,
    ) -> Result<(), UpdateBoardError> {
        let outcome: i64 = match outcome {
            Outcome::Draw => GameOutcome::Draw,
            Outcome::Decisive {
                winner: Color::White,
            } => GameOutcome::WhiteWin,
            Outcome::Decisive {
                winner: Color::Black,
            } => GameOutcome::BlackWin,
        }
        .into();

        sqlx::query!(
            "
            UPDATE games
            SET outcome = $1
            WHERE id = $2
            ",
            outcome,
            game_info.id
        )
        .execute(&self.pool)
        .change_context(UpdateBoardError::DatabaseError)
        .await?;

        Ok(())
    }

    async fn update_board(&self, info: &mut GameInfo<'_>) -> Result<(), UpdateBoardError> {
        // update db with new game info
        match info.last_move {
            MoveStatus::Move(game_move) => self.add_move_to_db(info, game_move).await?,
            MoveStatus::Check(game_move) => self.add_move_to_db(info, game_move).await?,
            MoveStatus::Checkmate(game_move) => {
                // if it's checkmate, it means the last player won
                let winner = info.current_player.other();
                self.add_move_to_db(info, game_move).await?;
                self.set_game_outcome(info, Outcome::Decisive { winner })
                    .await?;
            }
            MoveStatus::Stalemate(game_move) => {
                self.add_move_to_db(info, game_move).await?;
                self.set_game_outcome(info, Outcome::Draw).await?;
            }
            MoveStatus::Draw => self.set_game_outcome(info, Outcome::Draw).await?,
            MoveStatus::DrawOffer(_) => {}
            MoveStatus::GameStart => {}
        }

        info.black
            .update_board(
                info.white.username(),
                &Color::Black,
                &info.board,
                &info.last_move,
                info.current_player == Color::Black,
            )
            .await?;

        info.white
            .update_board(
                info.black.username(),
                &Color::White,
                &info.board,
                &info.last_move,
                info.current_player == Color::White,
            )
            .await
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
        &self,
        player: PlayerPlatform,
        san: &str,
    ) -> Result<MoveStatus, ChessError> {
        let Some(mut game) = self.get_game(&player) else {
            bail!(ChessError::InvalidPlayer)
        };

        let (mut white, mut black) = game.key().to_owned();
        let current_player_color = Color::from_white(player == *white.platform());

        if san == DRAW_OFFER_SAN {
            return self
                .handle_draw(game, current_player_color, &mut white, &mut black)
                .await;
        }

        let san: San = san
            .parse::<San>()
            .attach_printable("Failed to parse SAN")
            .change_context(ChessError::InvalidMove)?;

        let chess_move = game.play_move(&current_player_color, san)?;
        let current_player = current_player_color.other();

        let move_status = if let Some(outcome) = game.outcome() {
            match outcome {
                Outcome::Draw => MoveStatus::Stalemate(chess_move),
                Outcome::Decisive { .. } => MoveStatus::Checkmate(chess_move),
            }
        } else if game.is_check() {
            MoveStatus::Check(chess_move)
        } else {
            MoveStatus::Move(chess_move)
        };

        let mut game_info = GameInfo {
            id: game.id(),
            white: &mut white,
            black: &mut black,
            last_move: &move_status,
            current_player,
            board: game.board(),
            position: game.position(),
        };

        self.update_board(&mut game_info)
            .await
            .change_context(ChessError::DatabaseError)?;

        if let Some(outcome) = game.outcome() {
            let key = game.key().to_owned();
            drop(game);
            self.update_game_elo(&key, outcome).await?;
        }

        Ok(move_status)
    }

    async fn handle_draw(
        &self,
        mut game: Game<'_>,
        current_player_color: Color,
        white: &mut Player,
        black: &mut Player,
    ) -> Result<MoveStatus, ChessError> {
        if game.draw_offer(current_player_color) {
            let move_status = MoveStatus::Draw;

            let mut game_info = GameInfo {
                id: game.id(),
                white,
                black,
                last_move: &move_status,
                current_player: current_player_color,
                board: game.board(),
                position: game.position(),
            };

            self.update_board(&mut game_info)
                .await
                .change_context(ChessError::DatabaseError)?;

            let key = game.key().to_owned();
            drop(game);
            // don't update any ELOs, just remove the game
            self.current_games.remove(&key);
            Ok(MoveStatus::Draw)
        } else {
            let move_status = MoveStatus::DrawOffer(current_player_color);

            let mut game_info = GameInfo {
                id: game.id(),
                white,
                black,
                last_move: &move_status,
                current_player: current_player_color,
                board: game.board(),
                position: game.position(),
            };

            self.update_board(&mut game_info)
                .await
                .change_context(ChessError::DatabaseError)?;

            Ok(MoveStatus::DrawOffer(current_player_color))
        }
    }

    /// Gets the game for a specific player on discord
    pub async fn find_player_discord(&self, id: UserId) -> Option<PlayerPlatform> {
        self.current_games.iter().find_map(|entry| {
            let (white, black) = entry.key();
            let white = white.platform();
            let black = black.platform();

            let player = match (white, black) {
                (PlayerPlatform::Discord { user, .. }, _) if user.id == id => Some(white),
                (_, PlayerPlatform::Discord { user, .. }) if user.id == id => Some(black),
                _ => None,
            }?;

            Some(player.clone())
        })
    }

    /// Gets the valid moves for the given player
    pub async fn get_moves(&self, player: &PlayerPlatform) -> SanArray {
        let Some(game) = self.get_game(player) else {
            return SanArray::new();
        };

        game.valid_moves_san()
    }

    async fn update_game_elo<'a>(
        &self,
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
}
