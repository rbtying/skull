use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::Player;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Initialize {
    players: Vec<Player>,
}
