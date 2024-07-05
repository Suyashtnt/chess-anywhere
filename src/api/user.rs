use std::fmt;

use aide::axum::{routing::get_with, ApiRouter, RouterExt};
use axum::{extract::State, http::StatusCode, Json, Router};
use axum_login::AuthUser;
use error_stack::{report, FutureExt};
use schemars::JsonSchema;
use serde::Serialize;

use super::{
    error::AxumReport,
    session::{AuthSession, InnerAuthSession},
    ApiState,
};

pub fn router() -> ApiRouter<ApiState> {
    Router::new().api_route(
        "/user/stats",
        get_with(stats, |op| {
            op.description("Get the stats of the logged in user")
                .security_requirement("Requires user login")
                .response::<401, Json<StatsError>>()
                .response::<500, Json<StatsError>>()
                .response::<200, Json<StatsResponse>>()
        }),
    )
}

#[derive(Debug, Serialize, JsonSchema)]
struct StatsResponse {
    username: String,
    elo: f64,
}

#[derive(Debug, JsonSchema)]
/// Error response for the stats endpoint
enum StatsError {
    /// You need to login to see your stats
    Unauthorized,
    /// Internal Backend error
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
    let user = InnerAuthSession::from(session).user.ok_or(AxumReport::new(
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
