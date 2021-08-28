use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Player, PlayerID, Score};

/// The set of players playing the game.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Players {
    /// The ordered list of player IDs, used to determine the next player.
    player_ids: Vec<PlayerID>,
    /// The storage for player-state. Note that the player's ID is replicated
    /// inside the map -- the redundancy of using `player_ids` rather than a
    /// separate map is to ensure that ordering is not lost after ser/de.
    players: HashMap<PlayerID, Player>,
    /// Observers are not participating in the game -- they can only observe.
    observers: Vec<Player>,
    /// A holding area for the ID to be allocated to the next player.
    next_player_id: PlayerID,
}

impl Players {
    pub fn new() -> Self {
        Self {
            player_ids: vec![],
            players: HashMap::new(),
            observers: vec![],
            next_player_id: PlayerID(1),
        }
    }

    pub fn player_ids(&self) -> &'_ [PlayerID] {
        &self.player_ids
    }

    /// Get all of the players which are currently in the game, in play order.
    pub fn players(&self) -> impl Iterator<Item = &'_ Player> {
        self.player_ids
            .iter()
            .flat_map(move |id| self.players.get(&id))
    }

    /// Get all of the players which are not currently in the game, in arbitrary order.
    pub fn observers(&self) -> impl Iterator<Item = &'_ Player> {
        self.observers.iter()
    }

    /// Get the player after the provided `player_id`. Returns `None` if the
    /// player is not found or the next player does not exist.
    pub fn next_player(&self, player_id: PlayerID) -> Option<&'_ Player> {
        let index = self.player_ids.iter().position(|p| *p == player_id)?;
        let next_player = self.player_ids[(index + 1) % self.players.len()];
        self.players.get(&next_player)
    }

    /// Get the (playing) player by PlayerID. Returns `PlayerDoesntExist` if not
    /// found, including if the player is currently observing.
    pub fn player(&self, id: PlayerID) -> Result<&'_ Player, PlayerError> {
        self.players.get(&id).ok_or(PlayerError::PlayerDoesntExist)
    }

    /// Add a player to the game (by name), returning the new `Players` and
    /// corresponding `PlayerID`. If the player was already playing, returns the
    /// preexisting player ID.
    pub fn add_player(&self, name: String) -> Result<(Self, PlayerID), PlayerError> {
        if name.len() > 128 {
            return Err(PlayerError::PlayerNameTooLong);
        }

        match self.players.values().find(|p| p.name == name) {
            Some(p) => Ok((self.clone(), p.player_id)),
            None => {
                let mut self_ = self.clone();
                self_.players.insert(
                    self.next_player_id,
                    Player {
                        name,
                        player_id: self.next_player_id,
                        score: Score::Zero,
                    },
                );
                self_.player_ids.push(self.next_player_id);
                self_.next_player_id = PlayerID(self.next_player_id.0 + 1);
                Ok((self_, self.next_player_id))
            }
        }
    }

    /// Remove a player from the game and from observation.
    pub fn remove_player(&self, player_id: PlayerID) -> Result<Self, PlayerError> {
        let idx = self
            .player_ids
            .iter()
            .position(|p| *p == player_id)
            .ok_or(PlayerError::PlayerDoesntExist)?;
        let mut self_ = self.clone();
        self_.player_ids.remove(idx);
        self_.players.remove(&player_id);
        if let Some(observer_idx) = self_
            .observers
            .iter()
            .position(|p| p.player_id == player_id)
        {
            self_.observers.remove(observer_idx);
        }

        Ok(self_)
    }

    /// Change the order of the players who are playing the game. The
    /// `reordered_player_ids` must be a permutation of the existing
    /// `player_ids`.
    pub fn reorder_players(
        &self,
        reordered_player_ids: Vec<PlayerID>,
    ) -> Result<Self, PlayerError> {
        let mut sorted_existing_player_ids = self.player_ids.clone();
        sorted_existing_player_ids.sort_by_key(|pid| pid.0);
        let mut sorted_reordered_player_ids = reordered_player_ids.clone();
        sorted_reordered_player_ids.sort_by_key(|pid| pid.0);
        if sorted_existing_player_ids != sorted_reordered_player_ids {
            Err(PlayerError::MismatchedPlayerIDs)
        } else {
            let mut self_ = self.clone();
            self_.player_ids = reordered_player_ids;
            Ok(self_)
        }
    }

    /// Convert the provided `player_id` into an observer rather than a player.
    pub fn make_player_into_observer(&self, player_id: PlayerID) -> Result<Self, PlayerError> {
        let mut self_ = self.clone();
        let player = self_
            .players
            .remove(&player_id)
            .ok_or(PlayerError::PlayerDoesntExist)?;
        self_.player_ids.retain(|p| *p != player_id);
        self_.observers.push(player);
        Ok(self_)
    }

    /// Convert the provided `player_id` into a player rather than an observer.
    pub fn make_observer_into_player(&self, player_id: PlayerID) -> Result<Self, PlayerError> {
        let mut self_ = self.clone();
        let idx = self_
            .observers
            .iter()
            .position(|p| p.player_id == player_id)
            .ok_or(PlayerError::PlayerDoesntExist)?;
        let player = self_.observers.remove(idx);
        self_.players.insert(player_id, player);
        self_.player_ids.push(player_id);
        Ok(self_)
    }

    /// Increment the score for the provided player. If a player just won the
    /// game, returns the winning player as well.
    pub fn increment_score(
        &self,
        player_id: PlayerID,
    ) -> Result<(Self, Option<PlayerID>), PlayerError> {
        let mut self_ = self.clone();
        let num_winners = self_
            .players
            .values()
            .map(|p| p.score)
            .filter(|s| *s == Score::WonGame)
            .count();
        let mut p = self_
            .players
            .get_mut(&player_id)
            .ok_or(PlayerError::PlayerDoesntExist)?;

        p.score = match p.score {
            Score::Zero => Score::WonOne,
            // Before declaring victory, make sure nobody else has already declared victory.
            Score::WonOne if num_winners == 0 => Score::WonGame,
            Score::WonOne | Score::WonGame => return Err(PlayerError::PlayerAlreadyWon),
        };

        let winning_player_id = if p.score == Score::WonGame {
            Some(p.player_id)
        } else {
            None
        };

        Ok((self_, winning_player_id))
    }

    /// Reset all scores (for players and observers) to zero.
    pub fn reset_all_scores(&self) -> Self {
        let mut self_ = self.clone();
        for p in self_.players.values_mut() {
            p.score = Score::Zero;
        }
        for o in self_.observers.iter_mut() {
            o.score = Score::Zero;
        }
        self_
    }
}

#[derive(Error, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PlayerError {
    #[error("Player does not exist")]
    PlayerDoesntExist,
    #[error("Need at least two players to start the game")]
    NotEnoughPlayers,
    #[error("Player name is too long")]
    PlayerNameTooLong,
    #[error("Reordered player IDs don't match existing")]
    MismatchedPlayerIDs,
    #[error("Player has already won the game!")]
    PlayerAlreadyWon,
}

#[cfg(test)]
mod tests {
    use super::super::{Player, PlayerID, Score};
    use super::Players;
}
