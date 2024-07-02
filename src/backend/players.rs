use std::{fmt, hash::Hash, sync::Arc};

use error_stack::{FutureExt, Report, Result};
use poise::serenity_prelude::{
    futures::TryFutureExt, CreateMessage, EditMessage, Http, Mentionable, Message,
    User as DiscordUser,
};
use shakmaty::{Board, Color};
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use sqlx::{types::BigDecimal, PgPool};
use uuid::Uuid;

use crate::{
    backend::chess::MoveStatus,
    discord::board::create_board_embed,
    users::{RawUser, User, UserService},
};

#[derive(Debug, Clone)]
pub enum PlayerPlatform {
    Discord {
        user: DiscordUser,
        game_message: Message,
        http: Arc<Http>,
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
        pool: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord { ref user, .. } => UserService::new(pool)
                .fetch_user_by_discord_id(user.id)
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
    pub async fn upsert(platform: PlayerPlatform, pool: &PgPool) -> Result<Self, sqlx::Error> {
        match Self::fetch(platform.clone(), pool).await? {
            Some(user) => Ok(user),
            None => Self::create(platform, pool).await,
        }
    }

    /// Creates a new user in the database
    pub async fn create(platform: PlayerPlatform, pool: &PgPool) -> Result<Self, sqlx::Error> {
        match platform {
            PlayerPlatform::Discord {
                user: ref discord_user,
                ..
            } => {
                let mut transaction = pool.begin().await?;
                let service = UserService::new(transaction.as_mut());

                let user = service.add_user(&discord_user.name).await?;
                service.attach_discord_id(&user, discord_user.id);

                transaction.commit().await?;

                Ok(Self { user, platform })
            }
        }
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
    pub async fn update_elo(
        &mut self,
        black: &mut Player,
        outcome: Outcomes,
        pool: &PgPool,
    ) -> Result<(), sqlx::Error> {
        let config = Glicko2Config::default();
        let (new_self, new_other) = glicko2(&self.elo, &black.elo, &outcome, &config);
        self.user.update_elo(new_self);
        black.user.update_elo(new_other);

        let mut transaction = pool.begin().await?;

        sqlx::query!(
            "
            UPDATE users
            SET elo_rating = $1, elo_deviation = $2, elo_volatility = $3
            WHERE id = $4
            ",
            self.user.elo().rating,
            self.user.elo().deviation,
            self.user.elo().volatility,
            self.user.id()
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE users
            SET elo_rating = $1, elo_deviation = $2, elo_volatility = $3
            WHERE id = $4
            ",
            black.user.elo().rating,
            black.user.elo().deviation,
            black.user.elo().volatility,
            black.user.id()
        )
        .execute(&mut *transaction)
        .await?;

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
                user,
            } => {
                let embed = create_board_embed(
                    self.user.username(),
                    other_player_name,
                    our_color,
                    board,
                    move_status,
                    is_our_turn,
                );

                // quick notif message
                game_message
                    .channel_id
                    .send_message(
                        &*http,
                        CreateMessage::new().content(format!("{}", user.mention())),
                    )
                    .change_context(UpdateBoardError::DiscordError)
                    .await?
                    .delete(&*http)
                    .change_context(UpdateBoardError::DiscordError)
                    .await?;

                game_message
                    .edit(&*http, EditMessage::default().content("").embed(embed))
                    .change_context(UpdateBoardError::DiscordError)
                    .await
            }
        }
    }

    pub fn platform(&self) -> &PlayerPlatform {
        &self.platform
    }

    pub fn username(&self) -> &str {
        self.user.username()
    }

    pub fn id(&self) -> Uuid {
        self.user.id()
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
