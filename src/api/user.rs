use std::fmt;

use aide::axum::{routing::get_with, ApiRouter, RouterExt};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json, Router,
};
use axum_login::AuthUser;
use error_stack::{report, FutureExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::users::GameOutcome;

use super::{
    error::AxumReport,
    session::{AuthSession, InnerAuthSession},
    ApiState,
};

pub fn router() -> ApiRouter<ApiState> {
    Router::new()
        .api_route(
            "/user/private/stats",
            get_with(stats_private, |op| {
                op.description(
                    "Get the stats of the logged in user (including non-public information)",
                )
                .security_requirement("Requires user login")
                .response::<401, Json<StatsError>>()
                .response::<500, Json<StatsError>>()
                .response::<200, Json<PrivateStatsResponse>>()
            }),
        )
        .api_route(
            "/user/stats",
            get_with(stats, |op| {
                op.description("Get the stats for the provided user")
                    .response::<400, Json<StatsError>>()
                    .response::<500, Json<StatsError>>()
                    .response::<200, Json<StatsResponse>>()
            }),
        )
}

#[derive(Debug, Serialize, JsonSchema)]
/// Response for the private stats endpoint
struct PrivateStatsResponse {
    /// The username of the user
    username: String,
    /// The elo rating of the user
    elo: f64,
    /// The number of games the user has won
    games_won: u64,
    /// The number of games the user has lost
    games_lost: u64,
    /// The number of games the user has drawn
    games_drawn: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
/// Response for the public stats endpoint
struct StatsResponse {
    /// The username of the user
    username: String,
    /// The elo rating of the user
    elo: f64,
}

#[derive(Debug, JsonSchema)]
/// Error response for the stats endpoint
enum StatsError {
    /// You need to login to see private stats
    Unauthorized,
    /// The user does not exist
    UserDoesNotExist,
    /// Internal Backend error
    BackendError,
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unauthorized => f.write_str("error.unauthorized"),
            Self::BackendError => f.write_str("error.internal"),
            Self::UserDoesNotExist => f.write_str("error.invalid.user"),
        }
    }
}

impl std::error::Error for StatsError {}

async fn stats_private(
    State(state): State<ApiState>,
    session: AuthSession,
) -> Result<Json<PrivateStatsResponse>, AxumReport<StatsError>> {
    let user = InnerAuthSession::from(session).user.ok_or(AxumReport::new(
        StatusCode::UNAUTHORIZED,
        report!(StatsError::Unauthorized),
    ))?;

    let db_user = state
        .get_user_by_id(user.id().id())
        .change_context(StatsError::BackendError)
        .await?
        .ok_or(AxumReport::new(
            // kinda weird that they're logged in but their account doesn't exist
            StatusCode::INTERNAL_SERVER_ERROR,
            report!(StatsError::UserDoesNotExist),
        ))?;

    let user_games = state
        .get_games_by_user_id(user.id().id())
        .change_context(StatsError::BackendError)
        .await?;

    let mut games_won = 0;
    let mut games_lost = 0;
    let mut games_drawn = 0;

    for game in user_games {
        match game.outcome() {
            Some(GameOutcome::Draw) => games_drawn += 1,
            Some(GameOutcome::WhiteWin) => {
                if game.white_id() == user.id().id() {
                    games_won += 1;
                } else {
                    games_lost += 1;
                }
            }
            Some(GameOutcome::BlackWin) => {
                if game.black_id() == user.id().id() {
                    games_won += 1;
                } else {
                    games_lost += 1;
                }
            }
            None => todo!(),
        }
    }

    Ok(Json(PrivateStatsResponse {
        username: db_user.username().to_string(),
        elo: db_user.elo().rating,
        games_drawn,
        games_lost,
        games_won,
    }))
}

#[derive(Debug, Deserialize, JsonSchema)]
/// Request parameters for the stats endpoint
struct StatsRequest {
    /// The username of the user to get stats for
    username: String,
}

async fn stats(
    State(state): State<ApiState>,
    Query(StatsRequest { username }): Query<StatsRequest>,
) -> Result<Json<StatsResponse>, AxumReport<StatsError>> {
    let db_user = state
        .get_user_by_username(&username)
        .change_context(StatsError::BackendError)
        .await?
        .ok_or(AxumReport::new(
            StatusCode::BAD_REQUEST,
            report!(StatsError::UserDoesNotExist),
        ))?;

    Ok(Json(StatsResponse {
        username: db_user.username().to_string(),
        elo: db_user.elo().rating,
    }))
}
