use std::fmt;

use aide::axum::{
    routing::{get, get_with, post, post_with},
    ApiRouter, RouterExt,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json, Router,
};
use base64::prelude::*;
use error_stack::{report, FutureExt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    error::AxumReport,
    session::{AuthSession, Credentials},
    ApiState,
};

pub fn router() -> ApiRouter<ApiState> {
    Router::new()
        .api_route(
            "/email/link",
            get_with(link_email, |op| {
                op.description(
                    "Link an email to a user account. This is a link sent to the user's email.",
                )
                .response::<401, Json<AuthError>>()
                .response::<500, Json<AuthError>>()
            }),
        )
        // TODO: get requests to these endpoints for pages
        .api_route("/email/login", post_with(login_email, |op|
            op.description("Starts the login process for a user with an email. They will receive a magic link in their email.")
                .response::<500, Json<AuthError>>()
        ))
        .api_route("/email/signup", post_with(signup_email, |op|
            op.description("Starts the signup process for a user with an email. They will receive a magic link in their email.")
                .response::<500, Json<AuthError>>()
        ))
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
/// Login with an email
struct EmailLogin {
    /// The email to login with
    email: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
/// Sign up with an email
struct EmailSignup {
    /// The username to sign up with
    username: String,
    /// The email to sign up with
    email: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
/// The query parameters for the email link
///
/// You should not need to generate this yourself. This comes by an email link
struct EmailLink {
    /// The magic link id
    id: i64,
    /// The magic link validation data
    entropy: String,
}

#[derive(Debug, JsonSchema)]
/// Error response for the authorization endpoints
enum AuthError {
    /// The email magic link is invalid. Could be expired or tampered with
    InvalidLink,
    /// Internal Backend error
    BackendError,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidLink => f.write_str("error.invalid.link"),
            Self::BackendError => f.write_str("error.internal"),
        }
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Serialize, JsonSchema)]
/// The response to an email request
pub enum EmailResponse {
    /// The email was sent
    EmailSent,
}

/// Start logging in a user with an email
async fn login_email(
    State(state): State<ApiState>,
    Json(EmailLogin { email }): Json<EmailLogin>,
) -> Result<Json<EmailResponse>, AxumReport<AuthError>> {
    let Some(user_id) = state
        .get_user_by_email(&email)
        .change_context(AuthError::BackendError)
        .await?
        .map(|user| user.id())
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
        .get_user_by_username(&username)
        .change_context(AuthError::BackendError)
        .await?
        .is_some()
    {
        return Ok(Json(EmailResponse::EmailSent));
    }

    let user_id = state
        .add_user(&username)
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
) -> Result<&'static str, AxumReport<AuthError>> {
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
