use std::fmt;

use aide::axum::{routing::get_with, ApiRouter, RouterExt};
use axum::{extract::State, http::StatusCode, Json, Router};
use axum_login::AuthUser;
use error_stack::{report, FutureExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend::chess::MoveStatus, BACKEND_SERVICE};

use super::{
    error::AxumReport,
    session::{AuthSession, InnerAuthSession},
    ApiState,
};

pub fn router() -> ApiRouter<ApiState> {
    Router::new().api_route(
        "/game/move",
        get_with(play_move, |op| {
            op.description("Play a move")
                .security_requirement("Requires user login")
                .security_requirement("Requires user to be in a game")
                .response::<400, Json<MoveError>>()
                .response::<401, Json<MoveError>>()
                .response::<500, Json<MoveError>>()
                .response::<200, Json<MoveResponse>>()
        }),
    )
}

#[derive(Debug, Serialize, JsonSchema)]
/// Chess side
enum Color {
    White,
    Black,
}

impl From<shakmaty::Color> for Color {
    fn from(color: shakmaty::Color) -> Self {
        match color {
            shakmaty::Color::White => Self::White,
            shakmaty::Color::Black => Self::Black,
        }
    }
}

#[derive(Debug, Serialize, JsonSchema)]
/// Response for any valid move played
enum MoveResponse {
    /// The move was played successfully
    ///
    /// Contains a string version of the move played
    MovePlayed(String),
    /// Your move put the opponent in check
    Check,
    /// Your move put the opponent in checkmate and won the game
    Checkmate,
    /// Your move put the game in a stalemate
    Stalemate,
    /// The game has started
    GameStart,
    /// The other player has offered a draw
    DrawOffer(Color),
    /// The other player has accepted your draw offer
    Draw,
}

impl From<MoveStatus> for MoveResponse {
    fn from(status: MoveStatus) -> Self {
        match status {
            MoveStatus::Move(chess_move) => Self::MovePlayed(chess_move.to_string()),
            MoveStatus::Check => Self::Check,
            MoveStatus::Checkmate => Self::Checkmate,
            MoveStatus::Stalemate => Self::Stalemate,
            MoveStatus::GameStart => Self::GameStart,
            MoveStatus::DrawOffer(color) => Self::DrawOffer(color.into()),
            MoveStatus::Draw => Self::Draw,
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct MoveRequest {
    san: String,
}

#[derive(Debug, JsonSchema)]
/// Error response for the move endpoint
enum MoveError {
    /// You need to be logged in to play a move/be in a game
    Unauthorized,
    /// Internal Backend error
    BackendError,
    /// You are not in a game
    NotInGame,
    /// An invalid move was played
    InvalidMove,
    /// The current move is not your turn
    NotYourTurn,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unauthorized => f.write_str("You need to login to see your stats"),
            Self::BackendError => f.write_str("Internal Backend error"),
            Self::NotInGame => f.write_str("You are not in a game"),
            Self::InvalidMove => f.write_str("An invalid move was played"),
            Self::NotYourTurn => f.write_str("It's not your turn to play a move"),
        }
    }
}

impl std::error::Error for MoveError {}

#[tracing::instrument]
pub async fn play_move(
    State(state): State<ApiState>,
    session: AuthSession,
    Json(MoveRequest { san }): Json<MoveRequest>,
) -> Result<Json<MoveResponse>, AxumReport<MoveError>> {
    let user = InnerAuthSession::from(session).user.ok_or(AxumReport::new(
        StatusCode::UNAUTHORIZED,
        report!(MoveError::Unauthorized),
    ))?;

    let player = crate::backend::players::PlayerPlatform::WebApi {
        user_id: user.id().id(),
    };

    let game = match BACKEND_SERVICE.get().unwrap().play_move(player, &san).await {
        Ok(game) => game,
        Err(err) => match err.current_context() {
            crate::backend::chess::ChessError::InvalidMove => {
                return Err(AxumReport::new(
                    StatusCode::BAD_REQUEST,
                    err.change_context(MoveError::InvalidMove),
                ))
            }
            crate::backend::chess::ChessError::NotYourTurn => {
                return Err(AxumReport::new(
                    StatusCode::BAD_REQUEST,
                    err.change_context(MoveError::NotYourTurn),
                ))
            }
            crate::backend::chess::ChessError::InvalidPlayer => {
                return Err(AxumReport::new(
                    StatusCode::BAD_REQUEST,
                    err.change_context(MoveError::NotInGame),
                ))
            }
            _ => return Err(err.change_context(MoveError::BackendError).into()),
        },
    };

    Ok(Json(game.into()))
}
