mod auth;
pub mod error;
pub mod user;

use base64::prelude::*;
use core::fmt;
use std::{
    future::{Future, IntoFuture},
    net::SocketAddr,
};

use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use error_stack::{FutureExt as ErrorFutureExt, Report, Result};
use poise::serenity_prelude::FutureExt;
use resend_rs::{types::CreateEmailBaseOptions, Resend};
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;

use crate::{backend::ServiceError, users::UserService};

/// An axum server exposing ways to play chess through an API
#[derive(Debug, Clone)]
pub struct ApiService {
    session_store: PostgresStore,
    pub state: ApiState,
}

#[derive(Debug, Clone)]
/// A more direct API to the database and email sending
pub struct ApiState {
    resend: Resend,
    pool: sqlx::PgPool,
}

#[derive(Debug)]
pub enum EmailError {
    SqlxError,
    ResendError,
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SqlxError => f.write_str("SQLx error"),
            Self::ResendError => f.write_str("Resend error"),
        }
    }
}

impl std::error::Error for EmailError {}

impl ApiState {
    pub async fn get_userid_by_username(
        &self,
        username: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        UserService::fetch_user_by_username(username, &self.pool)
            .await
            .map_err(Report::from)
            .map(|row| row.map(|row| row.id()))
    }

    pub async fn get_userid_by_email(&self, email: &str) -> Result<Option<Uuid>, sqlx::Error> {
        UserService::fetch_user_by_email(email, &self.pool)
            .await
            .map_err(Report::from)
            .map(|row| row.map(|row| row.id()))
    }

    pub async fn add_user(&self, username: &str) -> Result<Uuid, sqlx::Error> {
        UserService::create(username, &self.pool)
            .await
            .map(|row| row.id())
    }

    pub async fn send_magic_email(&self, email: &str, user_id: Uuid) -> Result<(), EmailError> {
        // silently ignore if email already exists
        if self
            .get_userid_by_email(&email)
            .change_context(EmailError::SqlxError)
            .await?
            .is_some()
        {
            return Ok(());
        };

        let entropy: Vec<u8> = (0..32).map(|_| rand::random()).collect();

        // convert entropy into a string
        let entropy_str = BASE64_URL_SAFE.encode(&entropy);

        let email_id = UserService::add_email_verification(user_id, email, &entropy, &self.pool)
            .change_context(EmailError::SqlxError)
            .await?;

        // TODO: proper email templating
        let body = format!(
            "
            Click the following link to log in:
            https://chess-anywhere.wobbl.in/email/link?id={}&entropy={}
            ",
            email_id, entropy_str
        );

        let email = CreateEmailBaseOptions::new(
            "no-reply@chess-anywhere.wobbl.in",
            [email],
            "Login to Chess Anywhere",
        )
        .with_text(&body);

        self.resend
            .emails
            .send(email)
            .change_context(EmailError::ResendError)
            .await?;

        Ok(())
    }
}

impl ApiService {
    pub async fn start<F>(
        resend_api_key: &str,
        pool: sqlx::PgPool,
        shutdown_signal: F,
        port: u16,
    ) -> Result<(Self, JoinHandle<Result<(), ServiceError>>), sqlx::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let resend = Resend::new(resend_api_key);

        let session_store = PostgresStore::new(pool.clone());
        session_store.migrate().await?;

        let api_state = ApiState {
            resend,
            pool: pool.clone(),
        };

        let task_session_store = session_store.clone();
        let task_pool = pool.clone();
        let task_api_state = api_state.clone();

        let task = tokio::task::spawn(async move {
            let deletion_task = tokio::task::spawn(
                task_session_store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            );

            let session_layer = SessionManagerLayer::new(task_session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1)));

            let tracing_layer = TraceLayer::new_for_http();

            let backend = user::Backend::new(task_pool.clone());
            let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

            let app = Router::new()
                .merge(auth::router())
                .with_state(task_api_state)
                .layer(tracing_layer)
                .layer(auth_layer);

            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

            axum::serve(listener, app.into_make_service())
                .with_graceful_shutdown(
                    shutdown_signal.then(|_| async move { deletion_task.abort() }),
                )
                .into_future()
                .change_context(ServiceError)
                .await
        });

        Ok((
            Self {
                session_store,
                state: api_state,
            },
            task,
        ))
    }
}
