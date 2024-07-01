use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use super::ApiState;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLogin {
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSignup {
    username: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailLink {
    id: String,
    entropy: String,
}

/// Login a user with an email
pub async fn login_email(
    State(state): State<ApiState>,
    session: Session,
    body: Json<EmailLogin>,
) -> impl IntoResponse {
    todo!()
}

/// Sign up a new user with an email
pub async fn signup_email(
    State(state): State<ApiState>,
    session: Session,
    body: Json<EmailSignup>,
) -> impl IntoResponse {
    todo!()
}

/// Links a user with an email via a magic link sent to their email
pub async fn link_email(
    State(state): State<ApiState>,
    session: Session,
    body: Json<EmailLink>,
) -> impl IntoResponse {
    todo!()
}
