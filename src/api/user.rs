use std::fmt;

use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend};
use uuid::Uuid;
use veil::Redact;

#[derive(Debug, Clone)]
pub enum Credentials {
    /// Logged in via an email magic link
    Email {
        /// The magic link id
        id: Uuid,
        /// The magic link data (AKA entropy garbage)
        data: Vec<u8>,
        /// The email address
        email: String,
    },
}

/// A logged in user to the web API
#[derive(Redact, Clone)]
pub struct User {
    id: Uuid,
    username: String,
    #[redact]
    login: Option<Credentials>,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        match &self.login {
            Some(Credentials::Email { data, .. }) => data,
            None => &[],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: sqlx::postgres::PgPool,
}

impl Backend {
    pub fn new(db: sqlx::postgres::PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match credentials {
            Credentials::Email { data, id, email } => {
                let record = sqlx::query!(
                    "
                        SELECT expiry_date, username, email
                        FROM email_verification
                        INNER JOIN users ON email_verification.user_id = users.id
                        WHERE
                            email_verification.id = $1 AND
                            email_verification.email = $2 AND
                            email_verification.data = $3 AND
                            email_verification.expiry_date > NOW()
                        ",
                    id,
                    email,
                    data
                )
                .fetch_optional(&self.db)
                .await?;

                Ok(record.map(|record| User {
                    id,
                    username: record.username,
                    login: Some(Credentials::Email { id, data, email }),
                }))
            }
        }
    }

    async fn get_user(
        &self,
        id: &<Self::User as AuthUser>::Id,
    ) -> Result<Option<Self::User>, Self::Error> {
        let record = sqlx::query!(
            "
                SELECT username
                FROM users
                WHERE id = $1
                ",
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(record.map(|record| User {
            id: *id,
            username: record.username,
            login: None,
        }))
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
