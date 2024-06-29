use std::{error::Error, fmt};

use arrayvec::ArrayVec;
use error_stack::{ensure, Result, ResultExt};
use replace_with::replace_with_or_abort_and_return;
use shakmaty::{san::San, Board, Chess, Color, Move, Outcome, Position};

pub type SanArray = ArrayVec<San, 256>;

#[derive(Debug)]
/// Lightweight wrapper around the shakmaty Chess board
pub struct ChessGame(Chess);

#[derive(Debug)]
pub enum ChessError {
    InvalidMove,
    NotYourTurn,
    GameOver,
    InvalidPlayer,
    DatabaseError,
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidMove => f.write_str("You played an invalid move!"),
            Self::NotYourTurn => f.write_str("It is not your turn!"),
            Self::GameOver => f.write_str("The game is over!"),
            Self::InvalidPlayer => f.write_str("Invalid player. You are probably not in the game!"),
            Self::DatabaseError => f.write_str("Uh oh, something went wrong with the database! Please try again or report the error."),
        }
    }
}

impl Error for ChessError {}

impl ChessGame {
    pub fn new() -> Self {
        Self(Chess::default())
    }

    /// Plays a move for the given player
    ///
    /// # Returns
    /// Returns Ok(chess_move) if the move was successful
    ///
    /// # Errors
    /// Errors if the move is invalid or it is not the player's turn
    pub fn play_move(&mut self, player_color: &Color, san: San) -> Result<Move, ChessError> {
        // just in case we didn't remove the game yet for some reason
        ensure!(!self.0.is_game_over(), ChessError::GameOver);

        let color_to_move = self.0.turn();
        ensure!(
            player_color.is_white() == color_to_move.is_white(),
            ChessError::NotYourTurn
        );

        let chess_move = san
            .to_move(&self.0)
            .attach_printable("Failed to convert SAN to move")
            .change_context(ChessError::InvalidMove)?;

        let was_successful =
            replace_with_or_abort_and_return(&mut self.0, |board| match board.play(&chess_move) {
                Ok(new_board) => (true, new_board),
                Err(err) => (false, err.into_inner()),
            });

        ensure!(was_successful, ChessError::InvalidMove);

        Ok(chess_move)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.0.outcome()
    }

    pub fn is_check(&self) -> bool {
        self.0.is_check()
    }

    /// Returns the SAN of the valid moves for the current player
    pub fn valid_moves_san(&self) -> SanArray {
        self.0
            .legal_moves()
            .into_iter()
            .map(|m| San::from_move(&self.0, &m))
            .collect()
    }

    /// Gets the current board
    pub fn board(&self) -> &Board {
        self.0.board()
    }
}
