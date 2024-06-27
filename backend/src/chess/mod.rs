use std::{error::Error, fmt};

use arrayvec::ArrayVec;
use error_stack::{ensure, Result, ResultExt};
use replace_with::replace_with_or_abort_and_return;
use shakmaty::{san::San, Chess, Position};

use crate::auth::Player;

pub type SanArray = ArrayVec<San, 256>;

#[derive(Debug)]
pub struct ChessGame {
    // db: sqlx::postgres::PgPool,
    white: Player,
    black: Player,
    board: Chess,
}

#[derive(Debug)]
pub enum ChessError {
    InvalidMove,
    NotYourTurn,
    GameOver,
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidMove => f.write_str("You played an invalid move!"),
            Self::NotYourTurn => f.write_str("It is not your turn!"),
            Self::GameOver => f.write_str("The game is over!"),
        }
    }
}

impl Error for ChessError {}

impl ChessGame {
    pub fn new(white: Player, black: Player) -> Self {
        Self {
            white,
            black,
            board: Chess::default(),
        }
    }

    /// Plays a move for the given player
    ///
    /// # Returns
    /// Returns Ok(true) if the game is over
    ///
    /// # Errors
    /// Errors if the move is invalid or it is not the player's turn
    pub fn play_move(&mut self, player: &Player, san: San) -> Result<bool, ChessError> {
        // just in case we didn't remove the game yet for some reason
        ensure!(!self.board.is_game_over(), ChessError::GameOver);

        let color_to_move = self.board.turn();
        let player_to_move = match color_to_move {
            shakmaty::Color::White => &self.white,
            shakmaty::Color::Black => &self.black,
        };

        ensure!(player_to_move == player, ChessError::NotYourTurn);

        let chess_move = san
            .to_move(&self.board)
            .attach_printable("Failed to convert SAN to move")
            .change_context(ChessError::InvalidMove)?;

        let was_successful = replace_with_or_abort_and_return(&mut self.board, |board| match board
            .play(&chess_move)
        {
            Ok(new_board) => (true, new_board),
            Err(err) => (false, err.into_inner()),
        });

        ensure!(was_successful, ChessError::InvalidMove);

        Ok(self.board.is_game_over())
    }

    /// Returns the SAN of the valid moves for the current player
    pub fn valid_moves_san(&self) -> SanArray {
        self.board
            .legal_moves()
            .into_iter()
            .map(|m| San::from_move(&self.board, &m))
            .collect()
    }
}
