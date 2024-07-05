use std::{fmt, hash::Hash, sync::Arc};

use error_stack::{FutureExt, Result};
use poise::serenity_prelude::{EditMessage, Http, Message, User as DiscordUser};
use shakmaty::{Board, Color};
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use sqlx::SqlitePool;

use crate::{
    backend::chess::MoveStatus,
    discord::board::create_board_embed,
    users::{User, UserService},
};

#[derive(Debug, Clone)]
pub enum PlayerPlatform {
    Discord {
        user: DiscordUser,
        game_message: Message,
        http: Arc<Http>,
    },
    WebApi {
        user_id: i64,
    },
}

impl PartialEq for PlayerPlatform {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Discord { user: a, .. }, Self::Discord { user: b, .. }) => a == b,
            (Self::WebApi { user_id: a }, Self::WebApi { user_id: b }) => a == b,
            _ => false,
        }
    }
}

impl Eq for PlayerPlatform {}

#[derive(Debug, Clone)]
/// A high-level currently playing player API
pub struct Player {
    user: User,
    /// The platform the player is currently playing on
    platform: PlayerPlatform,
}

impl Player {
    /// Gets a user from the database based on their current platform
    pub async fn fetch(
        platform: PlayerPlatform,
        pool: &SqlitePool,
    ) -> Result<Option<Self>, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord { ref user, .. } => {
                UserService::fetch_user_by_discord_id(user.id, pool)
                    .await
                    .map(|row| {
                        row.map(|row| Self {
                            user: row.into(),
                            platform,
                        })
                    })
            }
            PlayerPlatform::WebApi { user_id } => UserService::fetch_user_by_id(user_id, pool)
                .await
                .map(|row| {
                    row.map(|row| Self {
                        user: row.into(),
                        platform,
                    })
                }),
        }
    }

    /// Gets a user from the database based on their current platform,
    /// or creates them if they're not in the database
    /// using the provided closure to create the user
    pub async fn upsert(platform: PlayerPlatform, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
        match Self::fetch(platform.clone(), pool).await? {
            Some(user) => Ok(user),
            None => Self::create(platform, pool).await,
        }
    }

    /// Creates a new user in the database
    pub async fn create(platform: PlayerPlatform, pool: &SqlitePool) -> Result<Self, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord {
                user: ref discord_user,
                ..
            } => {
                let mut transaction = pool.begin().await?;

                let user = UserService::create(&discord_user.name, &mut *transaction).await?;
                user.attach_discord_id(discord_user.id, &mut *transaction)
                    .await?;

                transaction.commit().await?;

                Ok(Self { user, platform })
            }
            // how the hell would one create the user by starting the game... and provide an arbitrary user_id?
            PlayerPlatform::WebApi { .. } => {
                unreachable!("Cannot create a user from a WebApi")
            }
        }
    }

    /// Updates the ELO of the current player and the other player.
    ///
    /// # Arguments
    /// * `black` - The other player (black).
    /// * `outcome` - The outcome of the game. This is from the perspective of the current player (white).
    pub async fn update_elo(
        &mut self,
        black: &mut Player,
        outcome: Outcomes,
        pool: &SqlitePool,
    ) -> Result<(), sqlx::Error> {
        let config = Glicko2Config::default();
        let (new_self, new_other) = glicko2(&self.elo(), &black.elo(), &outcome, &config);

        let mut transaction = pool.begin().await?;

        self.user.update_elo(new_self, &mut *transaction).await?;
        black.user.update_elo(new_other, &mut *transaction).await?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn update_board(
        &mut self,
        other_player_name: &str,
        our_color: &Color,
        board: &Board,
        move_status: &MoveStatus,
        is_our_turn: bool,
    ) -> Result<(), UpdateBoardError> {
        match &mut self.platform {
            PlayerPlatform::Discord {
                ref mut game_message,
                http,
                ..
            } => {
                let embed = create_board_embed(
                    self.user.username(),
                    other_player_name,
                    our_color,
                    board,
                    move_status,
                    is_our_turn,
                );

                game_message
                    .edit(&*http, EditMessage::default().content("").embed(embed))
                    .change_context(UpdateBoardError::DiscordError)
                    .await
            }
            PlayerPlatform::WebApi { user_id } => {
                todo!("Implement sending an event for WebApi players via the API service")
            }
        }
    }

    pub fn platform(&self) -> &PlayerPlatform {
        &self.platform
    }

    pub fn username(&self) -> &str {
        self.user.username()
    }

    pub fn id(&self) -> i64 {
        self.user.id()
    }

    pub fn elo(&self) -> Glicko2Rating {
        self.user.elo()
    }
}

#[derive(Debug)]
pub enum UpdateBoardError {
    DatabaseError,
    DiscordError,
}

impl fmt::Display for UpdateBoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DiscordError => f.write_str("Failed to update the board on Discord"),
            Self::DatabaseError => f.write_str("Failed to update the board in the database"),
        }
    }
}

impl std::error::Error for UpdateBoardError {}

impl Hash for Player {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Player {}
