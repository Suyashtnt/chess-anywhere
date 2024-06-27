use std::hash::Hash;

use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq)]
pub enum PlayerPlatform {
    Discord(poise::serenity_prelude::UserId),
}

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
    /// Gets a user from the database if they're in it
    pub fn fetch(
        platform: PlayerPlatform,
        pool: &sqlx::postgres::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
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
}

impl Player {
    pub fn update_elo(&mut self, other: &mut Player, outcome: Outcomes) {
        let config = Glicko2Config::default();
        let (new_self, new_other) = glicko2(&self.elo, &other.elo, &outcome, &config);
        self.elo = new_self;
        other.elo = new_other;
    }
}

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
