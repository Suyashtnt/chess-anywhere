use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use aide::OperationInput;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_login::{AuthUser, AuthnBackend};
use serde::{Deserialize, Serialize};
use tracing::debug;
use veil::Redact;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    /// Logged in via an email magic link
    Email {
        /// The magic link id
        id: i64,
        /// The magic link data (AKA entropy garbage)
        data: Vec<u8>,
    },
}

impl fmt::Display for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Credentials::Email { id, .. } => write!(f, "email:{}", id),
        }
    }
}

/// A logged in user to the web API
#[derive(Redact, Clone)]
pub struct User {
    id: i64,
    username: String,
    #[redact]
    login: Credentials,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserId {
    id: i64,
    method: Credentials,
}

impl UserId {
    pub fn id(&self) -> i64 {
        self.id
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.id, self.method)
    }
}

impl AuthUser for User {
    type Id = UserId;

    fn id(&self) -> Self::Id {
        UserId {
            id: self.id,
            method: self.login.clone(),
        }
    }

    fn session_auth_hash(&self) -> &[u8] {
        match &self.login {
            Credentials::Email { data, .. } => data,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: sqlx::SqlitePool,
}

impl Backend {
    pub fn new(db: sqlx::SqlitePool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    #[tracing::instrument]
    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        debug!("Authenticating user");
        match credentials {
            Credentials::Email { data, id } => {
                let record = sqlx::query!(
                    "
                        SELECT expiry_date, username, email, user_id
                        FROM email_verification
                        INNER JOIN users ON email_verification.user_id = users.id
                        WHERE
                            email_verification.id = $1 AND
                            email_verification.data = $2 AND
                            email_verification.expiry_date >= unixepoch('now') AND
                            email_verification.used = FALSE
                        ",
                    id,
                    data
                )
                .fetch_optional(&self.db)
                .await?;

                // store in email_login if the record doesn't exist there yet
                if let Some(record) = record {
                    let mut transaction = self.db.begin().await?;

                    sqlx::query!(
                        "
                            UPDATE email_verification
                            SET used = TRUE
                            WHERE id = $1
                        ",
                        id
                    )
                    .execute(&mut *transaction)
                    .await?;

                    sqlx::query!(
                        "
                            INSERT INTO email_login (email, user_id)
                            VALUES ($1, $2)
                            ON CONFLICT (email) DO NOTHING
                            ",
                        record.email,
                        record.user_id
                    )
                    .execute(&mut *transaction)
                    .await?;

                    transaction.commit().await?;

                    Ok(Some(User {
                        id: record.user_id,
                        username: record.username,
                        login: Credentials::Email {
                            id: record.user_id,
                            data,
                        },
                    }))
                } else {
                    Ok(None)
                }
            }
        }
    }

    #[tracing::instrument]
    async fn get_user(
        &self,
        id: &<Self::User as AuthUser>::Id,
    ) -> Result<Option<Self::User>, Self::Error> {
        debug!("Getting user by id: {}", id);

        match &id.method {
            Credentials::Email { id: email_id, data } => {
                let record = sqlx::query!(
                    "
                        SELECT username
                        FROM users
                        LEFT JOIN email_verification ON users.id = email_verification.user_id
                        WHERE users.id = $1 AND
                        email_verification.id = $2
                        ",
                    id.id,
                    email_id,
                )
                .fetch_optional(&self.db)
                .await?;

                Ok(record.map(|record| User {
                    id: id.id,
                    username: record.username,
                    login: Credentials::Email {
                        id: *email_id,
                        data: data.clone(),
                    },
                }))
            }
        }
    }
}

pub type InnerAuthSession = axum_login::AuthSession<Backend>;

pub struct AuthSession(axum_login::AuthSession<Backend>);

#[async_trait]
impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    Backend: AuthnBackend + Send + Sync + 'static,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<axum_login::AuthSession<_>>()
            .cloned()
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Can't extract auth session. Is `AuthManagerLayer` enabled?",
            ))
            .map(AuthSession)
    }
}

impl OperationInput for AuthSession {}

impl From<AuthSession> for InnerAuthSession {
    fn from(session: AuthSession) -> Self {
        session.0
    }
}

impl Deref for AuthSession {
    type Target = axum_login::AuthSession<Backend>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AuthSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
