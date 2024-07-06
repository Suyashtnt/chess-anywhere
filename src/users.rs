// TODO: refactor app to depend on this for getting users from the database

use error_stack::Result;
use poise::serenity_prelude::UserId;
use skillratings::glicko2::Glicko2Rating;
use sqlx::{Executor, Sqlite};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct UserService;
impl UserService {
    pub async fn fetch_user_by_id(
        id: i64,
        executor: impl Executor<'_, Database = Sqlite>,
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
        executor: impl Executor<'_, Database = Sqlite>,
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
        executor: impl Executor<'_, Database = Sqlite>,
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

    pub async fn fetch_games_by_user_id(
        id: i64,
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<Vec<Game>, sqlx::Error> {
        sqlx::query_as!(
            RawGame,
            "
            SELECT id, white_id, black_id, outcome, created_at
            FROM games
            WHERE white_id = $1 OR black_id = $1
            ",
            id
        )
        .fetch_all(executor)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(Into::into)
    }

    pub async fn fetch_user_by_discord_id(
        id: UserId,
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<Option<User>, sqlx::Error> {
        let id = id.get() as i64;
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
        id: i64,
        executor: impl Executor<'_, Database = Sqlite>,
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
        executor: impl Executor<'_, Database = Sqlite>,
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
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<(), sqlx::Error> {
        let id = discord_id.get() as i64;
        let user_id = user.id();
        sqlx::query!(
            "
            INSERT INTO discord_id (discord_id, user_id)
            VALUES ($1, $2)
            ",
            id,
            user_id
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    pub async fn add_email_verification(
        user_id: i64,
        email: &str,
        data: &[u8],
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query!(
            "
            INSERT INTO email_verification (user_id, email, data, expiry_date)
            VALUES ($1, $2, $3, unixepoch('now', '+1 hour'))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Draw = 0,
    WhiteWin = 1,
    BlackWin = 2,
}

impl From<GameOutcome> for i64 {
    fn from(status: GameOutcome) -> Self {
        status as i64
    }
}

impl From<i64> for GameOutcome {
    fn from(status: i64) -> Self {
        match status {
            0 => Self::Draw,
            1 => Self::WhiteWin,
            2 => Self::BlackWin,
            _ => panic!("Invalid game outcome"),
        }
    }
}

/// A game a user has played
#[derive(Debug, Clone)]
pub struct Game {
    id: i64,
    white_id: i64,
    black_id: i64,
    outcome: Option<GameOutcome>,
    created_at: OffsetDateTime,
}

impl Game {
    #[must_use]
    pub const fn id(&self) -> i64 {
        self.id
    }

    #[must_use]
    pub const fn white_id(&self) -> i64 {
        self.white_id
    }

    #[must_use]
    pub const fn black_id(&self) -> i64 {
        self.black_id
    }

    #[must_use]
    pub const fn outcome(&self) -> Option<GameOutcome> {
        self.outcome
    }

    #[must_use]
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}

#[derive(Debug, Clone)]
pub struct User {
    /// ID of the player
    ///
    /// Unique
    id: i64,
    /// The username of the player
    ///
    /// Unique
    username: String,
    /// The ELO rating of the player
    elo: Glicko2Rating,
}

impl User {
    #[must_use]
    pub const fn new_with_rating(id: i64, username: String, elo: Glicko2Rating) -> Self {
        Self { id, username, elo }
    }

    #[must_use]
    pub const fn id(&self) -> i64 {
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
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<(), sqlx::Error> {
        self.elo = new_elo;
        sqlx::query!(
            "
            UPDATE users
            SET elo_rating = $1, elo_deviation = $2, elo_volatility = $3
            WHERE id = $4
            ",
            self.elo.rating,
            self.elo.deviation,
            self.elo.volatility,
            self.id
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    pub async fn attach_discord_id(
        &self,
        discord_id: UserId,
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<(), sqlx::Error> {
        UserService::attach_discord_id(self, discord_id, executor).await
    }

    pub async fn update_username(
        &mut self,
        new_username: String,
        executor: impl Executor<'_, Database = Sqlite>,
    ) -> Result<(), sqlx::Error> {
        self.username = new_username;
        sqlx::query!(
            "
            UPDATE users
            SET username = $1
            WHERE id = $2
            ",
            self.username,
            self.id
        )
        .execute(executor)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }
}

/// A raw user from the database, intended to be immediately converted into a User
pub struct RawUser {
    pub id: i64,
    pub username: String,
    pub elo_rating: f64,
    pub elo_deviation: f64,
    pub elo_volatility: f64,
}

/// A raw game from the database, intended to be immediately converted into a Game
pub struct RawGame {
    pub id: i64,
    pub white_id: i64,
    pub black_id: i64,
    pub outcome: Option<i64>,
    pub created_at: i64,
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

impl From<RawGame> for Game {
    fn from(raw: RawGame) -> Self {
        Self {
            id: raw.id,
            white_id: raw.white_id,
            black_id: raw.black_id,
            outcome: raw.outcome.map(GameOutcome::from),
            created_at: OffsetDateTime::from_unix_timestamp(raw.created_at).unwrap(),
        }
    }
}
