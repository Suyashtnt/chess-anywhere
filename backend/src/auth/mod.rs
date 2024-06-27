use std::hash::Hash;

use skillratings::{
    glicko2::{glicko2, Glicko2Config, Glicko2Rating},
    Outcomes,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct Player {
    /// UUID of the player
    id: Uuid,
    /// The discord user ID of the player, if they are linked
    discord: Option<poise::serenity_prelude::UserId>,
    /// The ELO rating of the player
    elo: Glicko2Rating,
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
