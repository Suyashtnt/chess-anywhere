use std::{error::Error, fmt, iter::once};

use arrayvec::ArrayVec;
use error_stack::{ensure, Result, ResultExt};
use replace_with::replace_with_or_abort_and_return;
use shakmaty::{san::San, Board, Chess, Color, Move, Outcome, Position};

use super::DRAW_OFFER_SAN;

pub type SanArray = ArrayVec<String, 256>;

#[derive(Debug)]
/// Lightweight wrapper around the shakmaty Chess board
pub(super) struct ChessGame {
    game: Chess,
    draw_offer: Option<Color>,
}

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
        Self {
            game: Chess::default(),
            draw_offer: None,
        }
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
        ensure!(!self.game.is_game_over(), ChessError::GameOver);

        let color_to_move = self.game.turn();
        ensure!(
            player_color.is_white() == color_to_move.is_white(),
            ChessError::NotYourTurn
        );

        let chess_move = san
            .to_move(&self.game)
            .attach_printable("Failed to convert SAN to move")
            .change_context(ChessError::InvalidMove)?;

        let was_successful = replace_with_or_abort_and_return(&mut self.game, |board| match board
            .play(&chess_move)
        {
            Ok(new_board) => (true, new_board),
            Err(err) => (false, err.into_inner()),
        });

        ensure!(was_successful, ChessError::InvalidMove);

        // remove any draw offers
        self.draw_offer = None;

        Ok(chess_move)
    }

    /// Sets the draw offer for the current player
    ///
    /// # Returns
    /// Returns true if the draw offer was already set by the other player,
    /// and the game is therefore now a draw.
    pub fn draw_offer(&mut self, player_color: Color) -> bool {
        if self.draw_offer.is_some_and(|c| c != player_color) {
            true
        } else {
            self.draw_offer = Some(player_color);
            false
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.game.outcome()
    }

    pub fn is_check(&self) -> bool {
        self.game.is_check()
    }

    /// Returns the SAN of the valid moves for the current player
    ///
    /// This includes = for draw offers
    pub fn valid_moves_san(&self) -> SanArray {
        self.game
            .legal_moves()
            .into_iter()
            .map(|m| m.to_string())
            .chain(once(DRAW_OFFER_SAN.to_string()))
            .collect()
    }

    /// Gets the current board
    pub fn board(&self) -> &Board {
        self.game.board()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MoveStatus {
    Move(Move),
    Check,
    Checkmate,
    Stalemate,
    GameStart,
    DrawOffer(Color),
    Draw,
}
