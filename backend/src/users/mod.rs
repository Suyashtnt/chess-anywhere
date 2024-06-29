mod board_drawing;

use std::{fmt, hash::Hash};

use board_drawing::BoardDrawer;
use error_stack::{FutureExt, Report, Result};
use poise::serenity_prelude::{
    futures::TryFutureExt, CacheHttp, Context, CreateEmbed, CreateEmbedFooter, EditMessage,
    Message, User,
};
use shakmaty::{Board, Color, Move};
use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use sqlx::{types::BigDecimal, PgPool};
use uuid::Uuid;

pub use board_drawing::MoveStatus;

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

#[derive(Debug, Clone)]
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
    pub async fn create(platform: PlayerPlatform, pool: &PgPool) -> Result<Self, sqlx::Error> {
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
    pub fn delete(self, pool: &PgPool) -> Result<(), sqlx::Error> {
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
    pub async fn update_elo(
        &mut self,
        black: &mut Player,
        outcome: Outcomes,
        pool: &PgPool,
    ) -> Result<(), sqlx::Error> {
        let config = Glicko2Config::default();
        let (new_self, new_other) = glicko2(&self.elo, &black.elo, &outcome, &config);
        self.elo = new_self;
        black.elo = new_other;

        let mut transaction = pool.begin().await?;

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
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            "
            UPDATE users
            SET elo_rating = $1, elo_deviation = $2, elo_volatility = $3
            WHERE id = $4
            ",
            black.elo.rating,
            black.elo.deviation,
            black.elo.volatility,
            black.id
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }

    // TODO: move to discord crate
    pub async fn update_board(
        &mut self,
        board: &Board,
        turn: Color,
        move_status: MoveStatus,
        other_player_name: &str,
        is_our_turn: bool,
    ) -> Result<(), UpdateBoardError> {
        let checkmate = move_status == MoveStatus::Checkmate;
        let board_drawer = BoardDrawer::new(board, turn, move_status);

        match &mut self.platform {
            PlayerPlatform::Discord {
                ref mut game_message,
                context,
                ..
            } => {
                let mut embed = CreateEmbed::default()
                    .title(format!("{} vs {}", self.username, other_player_name))
                    .description(board_drawer.draw_discord())
                    .footer(CreateEmbedFooter::new(
                        "Run /move to make a move on Discord using SAN",
                    ));

                if checkmate {
                    let other_player = if is_our_turn {
                        other_player_name
                    } else {
                        &self.username
                    };

                    embed = embed.field("Winner", other_player, true);
                } else {
                    let current_player = if is_our_turn {
                        &self.username
                    } else {
                        other_player_name
                    };

                    embed = embed.field("Current player", current_player, true);
                }

                game_message
                    .edit(
                        context.http(),
                        EditMessage::default().content("").embed(embed),
                    )
                    .change_context(UpdateBoardError::DiscordError)
                    .await
            }
        }
    }

    pub fn platform(&self) -> &PlayerPlatform {
        &self.platform
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn id(&self) -> Uuid {
        self.id
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
