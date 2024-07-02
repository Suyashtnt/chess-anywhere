// TODO: refactor app to depend on this for getting users from the database

use std::fmt;

use error_stack::Result;
use poise::serenity_prelude::UserId;
use skillratings::glicko2::Glicko2Rating;
use sqlx::{types::BigDecimal, Executor, Postgres};
use uuid::Uuid;

pub struct UserService<E>(E);

impl<'e, 'c: 'e, E> fmt::Debug for UserService<E>
where
    E: Executor<'c, Database = Postgres> + fmt::Debug + 'e,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserService")
            .field("0", &self.0 as &dyn fmt::Debug)
            .finish()
    }
}

impl<'e, 'c: 'e, E> UserService<E>
where
    E: Executor<'c, Database = Postgres> + 'e,
{
    pub const fn new(db: E) -> Self {
        Self(db)
    }

    pub async fn fetch_user_by_id(self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            RawUser,
            "
            SELECT id, username, elo_rating, elo_deviation, elo_volatility
            FROM users
            WHERE id = $1
            ",
            id
        )
        .fetch_optional(self.0)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn fetch_user_by_discord_id(self, id: UserId) -> Result<Option<User>, sqlx::Error> {
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
        .fetch_optional(self.0)
        .await
        .map(|row| row.map(Into::into))
        .map_err(Into::into)
    }

    pub async fn delete_user(self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "
            DELETE FROM users
            WHERE id = $1
            ",
            id
        )
        .execute(self.0)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    pub async fn add_user(self, username: &str) -> Result<User, sqlx::Error> {
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
        .fetch_one(self.0)
        .await
        .map(Into::into)
        .map_err(Into::into)
    }

    pub async fn attach_discord_id(
        self,
        user: &User,
        discord_id: UserId,
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
        .execute(self.0)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }
}

impl<'e, 'c: 'e, E> Clone for UserService<E>
where
    E: Executor<'c, Database = Postgres> + Clone + 'e,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'e, 'c: 'e, E> Copy for UserService<E> where E: Executor<'c, Database = Postgres> + Copy + 'e {}

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

    pub fn update_elo(&mut self, new_elo: Glicko2Rating) {
        self.elo = new_elo;
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
