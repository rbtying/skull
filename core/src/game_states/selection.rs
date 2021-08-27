use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{Card, Hand, PlayerID, Players};

/// In the `Selection` phase, the `selector` (who has won the bid in the `Bidding` phase) must draw
/// cards. If they draw `goal` flowers, they win; otherwise, they lose. They are required to draw
/// their own cards first, after which the player-order is arbitrary.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Selection {
    players: Players,
    selector: PlayerID,
    goal: u8,
    found: u8,
    hands: HashMap<PlayerID, Hand>,
    cards: HashMap<PlayerID, Vec<Card>>,
}

impl Selection {
    #[must_use]
    pub fn new(
        selector: PlayerID,
        goal: u8,
        players: Players,
        cards: HashMap<PlayerID, Vec<Card>>,
        hands: HashMap<PlayerID, Hand>,
    ) -> Result<Self, ()> {
        if cards.len() > goal as usize {
            Err(())
        } else {
            Ok(Self {
                selector,
                goal,
                players,
                cards,
                hands,
                found: 0,
            })
        }
    }

    #[must_use]
    pub fn pick_card(self, from_player: PlayerID) -> Result<SelectionResult, SelectionError> {
        if self.selector != from_player
            && !self
                .cards
                .get(&self.selector)
                .map(|c| c.is_empty())
                .unwrap_or(false)
        {
            return Err(SelectionError::IncorrectDrawOrder);
        }
        let (card, cards) = self.draw_card(from_player)?;
        Ok(match card {
            Card::Skull => SelectionResult::Failed(from_player),
            Card::Flower if self.found + 1 == self.goal => SelectionResult::Complete(self.selector),
            Card::Flower => SelectionResult::More(Selection {
                found: self.found + 1,
                cards,
                ..self
            }),
        })
    }

    #[must_use]
    fn draw_card(
        &self,
        player_id: PlayerID,
    ) -> Result<(Card, HashMap<PlayerID, Vec<Card>>), DrawError> {
        if !self.cards.contains_key(&player_id) {
            Err(DrawError::PlayerDoesntExist)
        } else {
            let mut cards_ = self.cards.clone();
            match cards_.get_mut(&player_id) {
                Some(player_cards) => match player_cards.pop() {
                    Some(card) => Ok((card, cards_)),
                    None => Err(DrawError::PlayerDoesntExist),
                },
                None => Err(DrawError::PlayerDoesntExist),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum SelectionResult {
    Complete(PlayerID),
    More(Selection),
    Failed(PlayerID),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SelectionError {
    #[error("Incorrect draw order")]
    IncorrectDrawOrder,
    #[error("Couldn't get card: {0}")]
    DrawError(#[from] DrawError),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DrawError {
    #[error("That player doesn't exist")]
    PlayerDoesntExist,
    #[error("That player doesn't have any cards left")]
    NoCardsLeft,
}
