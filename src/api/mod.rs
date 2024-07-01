mod routes;

use std::{
    future::{Future, IntoFuture},
    net::SocketAddr,
};

use axum::{
    routing::{get, post},
    Router,
};
use error_stack::{FutureExt as ErrorFutureExt, Result};
use poise::serenity_prelude::FutureExt;
use resend_rs::Resend;
use tokio::task::JoinHandle;
use tower_http::trace::TraceLayer;
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;

use crate::backend::ServiceError;

/// An axum server exposing ways to play chess through an API
#[derive(Debug, Clone)]
pub struct ApiService {
    pool: sqlx::postgres::PgPool,
    session_store: PostgresStore,
}

#[derive(Debug, Clone)]
pub struct ApiState {
    resend: Resend,
    pool: sqlx::postgres::PgPool,
}

impl ApiService {
    pub async fn start<F>(
        resend_api_key: &str,
        pool: sqlx::postgres::PgPool,
        shutdown_signal: F,
        port: u16,
    ) -> Result<(Self, JoinHandle<Result<(), ServiceError>>), sqlx::Error>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let resend = Resend::new(resend_api_key);

        let session_store = PostgresStore::new(pool.clone());
        session_store.migrate().await?;

        let task_session_store = session_store.clone();
        let task_pool = pool.clone();
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

            let app = Router::new()
                .route("/email/link", get(routes::link_email))
                // TODO: get requests to these endpoints for pages
                .route("/email/login", post(routes::login_email))
                .route("/email/signup", post(routes::signup_email))
                .with_state(ApiState {
                    resend,
                    pool: task_pool,
                })
                .layer(tracing_layer)
                .layer(session_layer);

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
                pool,
                session_store,
            },
            task,
        ))
    }

    pub async fn run(&self) -> Result<(), ServiceError> {
        todo!()
    }
}
