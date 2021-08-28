use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{Player, PlayerID, Score};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Initialize {
    players: Vec<Player>,
}

impl Initialize {
    pub fn new() -> Self {
        Self { players: vec![] }
    }

    pub fn players(&self) -> &'_ [Player] {
        &self.players
    }
}
