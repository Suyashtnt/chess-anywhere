// TODO: refactor app to depend on this for getting users from the database

use error_stack::Result;
use poise::serenity_prelude::UserId;
use skillratings::glicko2::Glicko2Rating;
use sqlx::{types::BigDecimal, Executor, Postgres};
use uuid::Uuid;

#[derive(Debug)]
pub struct UserService;
impl UserService {
    pub async fn fetch_user_by_id(
        id: Uuid,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            RawUser,
            "
            SELECT id, username, elo_rating, elo_deviation, elo_volatility
            FROM users
            WHERE id = $1
            ",
            id
        )
        .fetch_optional(executor)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn fetch_user_by_username(
        username: &str,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            RawUser,
            "
            SELECT id, username, elo_rating, elo_deviation, elo_volatility
            FROM users
            WHERE username = $1
            ",
            username
        )
        .fetch_optional(executor)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn fetch_user_by_email(
        email: &str,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            RawUser,
            "
            SELECT id, username, elo_rating, elo_deviation, elo_volatility
            FROM email_login
            INNER JOIN users ON user_id = id
            WHERE email = $1
            LIMIT 1
            ",
            email
        )
        .fetch_optional(executor)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn fetch_user_by_discord_id(
        id: UserId,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<Option<User>, sqlx::Error> {
        let id = BigDecimal::from(id.get());
        sqlx::query_as!(
            RawUser,
            "
            SELECT id, username, elo_rating, elo_deviation, elo_volatility
            FROM discord_id
            INNER JOIN users ON user_id = id
            WHERE discord_id = $1
            LIMIT 1
            ",
            id
        )
        .fetch_optional(executor)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn delete_user(
        id: Uuid,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            DELETE FROM users
            WHERE id = $1
            ",
            id
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    pub async fn create(
        username: &str,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<User, sqlx::Error> {
        let Glicko2Rating {
            deviation,
            rating,
            volatility,
        } = Glicko2Rating::default();

        sqlx::query_as!(
            RawUser,
            "
            INSERT INTO users (username, elo_rating, elo_deviation, elo_volatility)
            VALUES ($1, $2, $3, $4)
            RETURNING id, username, elo_rating, elo_deviation, elo_volatility
            ",
            username,
            rating,
            deviation,
            volatility
        )
        .fetch_one(executor)
        .await
        .map(Into::into)
        .map_err(Into::into)
    }

    async fn attach_discord_id(
        user: &User,
        discord_id: UserId,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        let id = BigDecimal::from(discord_id.get());
        sqlx::query!(
            "
            INSERT INTO discord_id (discord_id, user_id)
            VALUES ($1, $2)
            ",
            id,
            user.id()
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn update_elo(
        user_id: Uuid,
        elo: Glicko2Rating,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            UPDATE users
            SET elo_rating = $1, elo_deviation = $2, elo_volatility = $3
            WHERE id = $4
            ",
            elo.rating,
            elo.deviation,
            elo.volatility,
            user_id
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    pub async fn add_email_verification(
        user_id: Uuid,
        email: &str,
        data: &[u8],
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<Uuid, sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO email_verification (user_id, email, data, expiry_date)
            VALUES ($1, $2, $3, NOW() + INTERVAL '1 day')
            RETURNING id
            ",
            user_id,
            email,
            data
        )
        .fetch_one(executor)
        .await
        .map(|row| row.id)
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    /// UUID of the player
    ///
    /// Unique
    id: Uuid,
    /// The username of the player
    ///
    /// Unique
    username: String,
    /// The ELO rating of the player
    elo: Glicko2Rating,
}

impl User {
    #[must_use]
    pub const fn new_with_rating(id: Uuid, username: String, elo: Glicko2Rating) -> Self {
        Self { id, username, elo }
    }

    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }

    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub const fn elo(&self) -> Glicko2Rating {
        self.elo
    }

    pub async fn update_elo(
        &mut self,
        new_elo: Glicko2Rating,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        self.elo = new_elo;
        UserService::update_elo(self.id, new_elo, executor).await
    }

    pub async fn attach_discord_id(
        &self,
        discord_id: UserId,
        executor: impl Executor<'_, Database = Postgres>,
    ) -> Result<(), sqlx::Error> {
        UserService::attach_discord_id(self, discord_id, executor).await
    }
}

/// A raw user from the database, intended to be immediately converted into a User
pub struct RawUser {
    pub id: Uuid,
    pub username: String,
    pub elo_rating: f64,
    pub elo_deviation: f64,
    pub elo_volatility: f64,
}

impl From<RawUser> for User {
    fn from(raw: RawUser) -> Self {
        Self {
            id: raw.id,
            username: raw.username,
            elo: Glicko2Rating {
                rating: raw.elo_rating,
                deviation: raw.elo_deviation,
                volatility: raw.elo_volatility,
            },
        }
    }
}
