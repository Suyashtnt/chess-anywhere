use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::ApiState;

pub fn router() -> Router<ApiState> {
    Router::new()
        .route("/email/link", get(link_email))
        // TODO: get requests to these endpoints for pages
        .route("/email/login", post(login_email))
        .route("/email/signup", post(signup_email))
}

#[derive(Debug, Serialize, Deserialize)]
struct EmailLogin {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmailSignup {
    username: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmailLink {
    id: String,
    entropy: String,
}

/// Start logging in a user with an email
async fn login_email(
    State(state): State<ApiState>,
    session: Session,
    body: Json<EmailLogin>,
) -> impl IntoResponse {
    todo!()
}

/// Sign up a new user with an email
async fn signup_email(
    State(state): State<ApiState>,
    session: Session,
    Json(EmailSignup { username, email }): Json<EmailSignup>,
) -> impl IntoResponse {
    todo!()
}

/// Links and logs in a user with an email via a magic link sent to their email
async fn link_email(
    State(state): State<ApiState>,
    session: Session,
    query: Query<EmailLink>,
) -> impl IntoResponse {
    todo!()
}
