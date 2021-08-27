use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game_states::bidding::{Bidding, BiddingError};
use crate::types::{Card, Hand, HandError, PlayerID, Players};

/// In the placement phase, each player (in order) must either place a card from their hand into
/// the `cards`, or make a nonzero bid (which would transition to the `Bidding` phase).
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Placement {
    players: Players,
    hands: HashMap<PlayerID, Hand>,
    cards: HashMap<PlayerID, Vec<Card>>,
    current_player: PlayerID,
}

impl Placement {
    #[must_use]
    pub fn place_card(&self, player_id: PlayerID, card: Card) -> Result<Placement, PlacementError> {
        let next_player = self
            .players
            .next_player(player_id)
            .map(|p| p.player_id)
            .ok_or(PlacementError::PlayerDoesntExist)?;

        let mut new_hands = self.hands.clone();

        let h = new_hands
            .remove(&player_id)
            .ok_or(PlacementError::OutOfCards)?;
        if let Some(new_h) = h.remove_card(card)? {
            new_hands.insert(player_id, new_h);
        }

        let mut new_cards = self.cards.clone();
        new_cards
            .entry(player_id)
            .or_insert_with(Vec::new)
            .push(card);

        Ok(Self {
            hands: new_hands,
            cards: new_cards,
            current_player: next_player,
            players: self.players.clone(),
        })
    }

    #[must_use]
    pub fn bid(&self, player_id: PlayerID, amount: u8) -> Result<Bidding, BiddingError> {
        Bidding::new(
            self.players.clone(),
            self.hands.clone(),
            self.cards.clone(),
            (player_id, amount),
        )
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PlacementError {
    #[error("That player doesn't exist")]
    PlayerDoesntExist,
    #[error("No cards remaining to place")]
    OutOfCards,
    #[error("Couldn't play card {0}")]
    HandError(#[from] HandError),
}
