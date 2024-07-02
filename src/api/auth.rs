use std::fmt;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::prelude::*;
use error_stack::{report, FutureExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    error::AxumReport,
    user::{AuthSession, Credentials},
    ApiState,
};

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
    id: Uuid,
    entropy: String,
}

#[derive(Debug)]
enum AuthError {
    InvalidLink,
    BackendError,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidLink => f.write_str("Invalid link"),
            Self::BackendError => f.write_str("Internal Backend error"),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Serialize)]
pub enum EmailResponse {
    EmailSent,
}

/// Start logging in a user with an email
async fn login_email(
    State(state): State<ApiState>,
    Json(EmailLogin { email }): Json<EmailLogin>,
) -> Result<Json<EmailResponse>, AxumReport<AuthError>> {
    let Some(user_id) = state
        .get_userid_by_email(&email)
        .change_context(AuthError::BackendError)
        .await?
    else {
        return Ok(Json(EmailResponse::EmailSent));
    };

    state
        .send_magic_email(&email, user_id)
        .change_context(AuthError::BackendError)
        .await?;

    Ok(Json(EmailResponse::EmailSent))
}

/// Sign up a new user with an email
async fn signup_email(
    State(state): State<ApiState>,
    Json(EmailSignup { username, email }): Json<EmailSignup>,
) -> Result<Json<EmailResponse>, AxumReport<AuthError>> {
    if state
        .get_userid_by_username(&username)
        .change_context(AuthError::BackendError)
        .await?
        .is_some()
    {
        return Ok(Json(EmailResponse::EmailSent));
    }

    let user_id = state
        .create_user(&username)
        .change_context(AuthError::BackendError)
        .await?;

    state
        .send_magic_email(&email, user_id)
        .change_context(AuthError::BackendError)
        .await?;

    Ok(Json(EmailResponse::EmailSent))
}

/// Links and logs in a user with an email via a magic link sent to their email
async fn link_email(
    mut session: AuthSession,
    Query(EmailLink { id, entropy }): Query<EmailLink>,
) -> Result<impl IntoResponse, AxumReport<AuthError>> {
    let data = BASE64_URL_SAFE.decode(entropy.as_bytes()).unwrap();

    let credentials = Credentials::Email { id, data };

    let user = match session
        .authenticate(credentials)
        .change_context(AuthError::BackendError)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AxumReport::new(
                StatusCode::UNAUTHORIZED,
                report!(AuthError::InvalidLink),
            ))
        }
        Err(e) => return Err(e.into()),
    };

    if session.login(&user).await.is_err() {
        return Err(AxumReport::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            report!(AuthError::BackendError),
        ));
    }

    // TODO: replace this with an html page
    Ok("Logged in!")
}
