use std::fmt;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use axum_login::AuthUser;
use error_stack::{report, FutureExt};
use serde::Serialize;

use super::{error::AxumReport, session::AuthSession, ApiState};

pub fn router() -> Router<ApiState> {
    Router::new().route("/user/stats", get(stats))
}

#[derive(Debug, Serialize)]
struct StatsResponse {
    username: String,
    elo: f64,
}

#[derive(Debug)]
enum StatsError {
    Unauthorized,
    BackendError,
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unauthorized => f.write_str("You need to login to see your stats"),
            Self::BackendError => f.write_str("Internal Backend error"),
        }
    }
}

impl std::error::Error for StatsError {}

async fn stats(
    State(state): State<ApiState>,
    session: AuthSession,
) -> Result<Json<StatsResponse>, AxumReport<StatsError>> {
    let user = session.user.ok_or(AxumReport::new(
        StatusCode::UNAUTHORIZED,
        report!(StatsError::Unauthorized),
    ))?;

    let db_user = state
        .get_user_by_id(user.id().id())
        .change_context(StatsError::BackendError)
        .await?
        .ok_or(AxumReport::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            report!(StatsError::BackendError),
        ))?;

    Ok(Json(StatsResponse {
        username: db_user.username().to_string(),
        elo: db_user.elo().rating,
    }))
}
