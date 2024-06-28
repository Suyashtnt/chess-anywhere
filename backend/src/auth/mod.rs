use std::{fmt, hash::Hash};

use error_stack::{FutureExt, Report, Result};
use poise::serenity_prelude::{
    futures::TryFutureExt, CacheHttp, Context, EditMessage, Message, User,
};
use shakmaty::Board;
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use sqlx::types::BigDecimal;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PlayerPlatform {
    Discord {
        user: User,
        game_message: Message,
        context: Context,
    },
}

impl PartialEq for PlayerPlatform {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Discord { user: a, .. }, Self::Discord { user: b, .. }) => a == b,
        }
    }
}

impl Eq for PlayerPlatform {}

#[derive(Debug)]
pub struct Player {
    /// UUID of the player
    ///
    /// Unique
    id: Uuid,
    /// The username of the player
    ///
    /// Unique
    username: String,
    /// The platform the player is currently playing on
    platform: PlayerPlatform,
    /// The ELO rating of the player
    elo: Glicko2Rating,
}

impl Player {
    /// Gets a user from the database based on their current platform
    pub async fn fetch(
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord { ref user, .. } => {
                let id = BigDecimal::from(user.id.get());
                sqlx::query!(
                    "
                    SELECT id, username, discord_id, elo_rating, elo_deviation, elo_volatility from users
                    WHERE discord_id = $1
                    LIMIT 1
                    ",
                    id
                )
                .fetch_optional(pool)
                .map_err(Report::from)
                .map_ok(|row| row.map(|row| Self {
                    id: row.id,
                    username: row.username,
                    platform,
                    elo: Glicko2Rating {
                        rating: row.elo_rating,
                        deviation: row.elo_deviation,
                        volatility: row.elo_volatility,
                    }
                }))
                .await
            }
        }
    }

    /// Gets a user from the database based on their current platform,
    /// or creates them if they're not in the database
    /// using the provided closure to create the user
    pub async fn upsert(
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Self, sqlx::Error> {
        match Self::fetch(platform.clone(), pool).await? {
            Some(user) => Ok(user),
            None => Self::create(platform, pool).await,
        }
    }

    /// Gets a user by their username
    pub fn fetch_by_username(
        username: String,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        todo!()
    }

    /// Creates a new user in the database
    pub async fn create(
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Self, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord { ref user, .. } => {
                let id = BigDecimal::from(user.id.get());
                let Glicko2Rating {
                    rating,
                    deviation,
                    volatility,
                } = Glicko2Rating::new();

                let record = sqlx::query!(
                    "
                        INSERT INTO users (username, discord_id, elo_rating, elo_deviation, elo_volatility)
                        VALUES ($1, $2, $3, $4, $5)
                        RETURNING id, username
                    ",
                    user.name,
                    id,
                    rating,
                    deviation,
                    volatility
                )
                .fetch_one(pool)
                .map_err(Report::from)
                .await?;

                Ok(Self {
                    id: record.id,
                    username: record.username,
                    platform,
                    elo: Glicko2Rating {
                        rating,
                        deviation,
                        volatility,
                    },
                })
            }
        }
    }

    /// Deletes a user from the database
    pub fn delete(self, pool: &sqlx::postgres::PgPool) -> Result<(), sqlx::Error> {
        todo!()
    }

    /// Links (or updates) a user to a new platform
    ///
    /// Authorization is not checked here, it is assumed that the user has already been validated to own
    /// both platform accounts
    pub fn link_new_platform(
        &mut self,
        new_platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<(), sqlx::Error> {
        todo!()
    }

    /// Updates the ELO of the current player and the other player.
    ///
    /// # Arguments
    /// * `black` - The other player (black).
    /// * `outcome` - The outcome of the game. This is from the perspective of the current player (white).
    pub fn update_elo(&mut self, black: &mut Player, outcome: Outcomes) {
        let config = Glicko2Config::default();
        let (new_self, new_other) = glicko2(&self.elo, &black.elo, &outcome, &config);
        self.elo = new_self;
        black.elo = new_other;
    }

    pub async fn update_board(&mut self, board: &Board) -> Result<(), UpdateBoardError> {
        match &mut self.platform {
            PlayerPlatform::Discord {
                ref mut game_message,
                context,
                ..
            } => {
                game_message
                    .edit(context.http(), EditMessage::default())
                    .change_context(UpdateBoardError::DiscordError)
                    .await
            }
        }
    }

    pub fn platform(&self) -> &PlayerPlatform {
        &self.platform
    }
}

#[derive(Debug)]
pub enum UpdateBoardError {
    DiscordError,
}

impl fmt::Display for UpdateBoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DiscordError => f.write_str("Failed to update the board on Discord"),
        }
    }
}

impl std::error::Error for UpdateBoardError {}

impl Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Player {}
