use std::{fmt, hash::Hash};

use error_stack::{FutureExt, Result};
use poise::serenity_prelude::{CacheHttp, Context, EditMessage, Message, UserId};
use shakmaty::Board;
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use uuid::Uuid;

#[derive(Debug)]
pub enum PlayerPlatform {
    Discord {
        user: UserId,
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
        todo!()
    }

    /// Gets a user from the database based on their current platform,
    /// or creates them if they're not in the database
    /// using the provided closure to create the user
    pub async fn upsert<F>(
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
        create_method: F,
    ) -> Result<Self, sqlx::Error>
    where
        F: FnOnce() -> Self,
    {
        todo!()
    }

    /// Gets a user by their username
    pub fn fetch_by_username(
        username: String,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        todo!()
    }

    /// Creates a new user in the database
    pub fn create(
        username: String,
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Self, sqlx::Error> {
        todo!()
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
