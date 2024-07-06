use std::fmt;

use aide::axum::{routing::get_with, ApiRouter, RouterExt};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
    Json, Router,
};
use axum_login::AuthUser;
use error_stack::{report, FutureExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::{error::AxumReport, session::AuthSession, ApiState};

#[derive(Debug, Serialize, JsonSchema)]
/// The error response for the Discord OAuth API
enum DiscordError {
    /// Something went wrong when communicating with Discord
    DiscordError,
    /// Something went wrong on the internal backend
    BackendError,
    /// The user is not logged in
    Unauthorized,
}

impl fmt::Display for DiscordError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DiscordError => f.write_str("error.discord"),
            Self::BackendError => f.write_str("error.internal"),
            Self::Unauthorized => f.write_str("error.unauthorized"),
        }
    }
}

impl std::error::Error for DiscordError {}

pub fn router() -> ApiRouter<ApiState> {
    Router::new()
        .api_route(
            "/user/oauth/discord",
            get_with(send_to_discord, |op| {
                op.description(
                    "Send the user to Discord to link their account. This will redirect them to Discord's OAuth page.",
                )
            }),
        )
        .api_route(
            "/user/oauth/discord_callback",
            get_with(discord_callback, |op| {
                op.description(
                    "Discord will redirect the user here after they have linked their account. This will save the user's Discord ID.",
                )
                .security_requirement("Requires user login")
            }),
        )
}

async fn send_to_discord(State(state): State<ApiState>) -> Redirect {
    todo!()
}

#[derive(Debug, Deserialize, JsonSchema)]
/// Query arguments for the Discord oauth callback
struct DiscordCallback {
    /// The code Discord gives us
    code: String,
    /// CSRF state
    state: String,
}

async fn discord_callback(
    Query(query): Query<DiscordCallback>,
    State(state): State<ApiState>,
    session: AuthSession,
) -> Result<String, AxumReport<DiscordError>> {
    // output string for now, in future HTML
    todo!()
}
