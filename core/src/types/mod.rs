//! Generic types used throughout the core codebase.

use serde::{Deserialize, Serialize};
use thiserror::Error;

mod players;

pub use players::Players;

/// A unique identifier for a player.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(transparent)]
pub struct PlayerID(pub u32);

/// Information tracked about a player throughout the game.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Player {
    pub(crate) player_id: PlayerID,
    pub(crate) name: String,
    pub(crate) score: Score,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Score {
    Zero,
    WonOne,
    WonGame,
}

/// A card in the game. Note: Cards don't carry whether they are visible or not.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Card {
    Flower,
    Skull,
}

/// The cards that remain in a player's hand. A player can have at most one skull card, and should
/// have at most four total cards. Their hand should never be empty (i.e. `Option::<Hand>::None`
/// should be used instead).
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Hand {
    num_cards: u8,
    has_skull: bool,
}

impl Hand {
    #[must_use]
    pub fn new() -> Self {
        Self {
            num_cards: 4,
            has_skull: true,
        }
    }

    pub fn num_flowers(self) -> usize {
        self.num_cards as usize - self.num_skulls()
    }

    pub fn num_skulls(self) -> usize {
        if self.has_skull {
            1
        } else {
            0
        }
    }

    pub fn num_cards(self) -> usize {
        self.num_cards as usize
    }

    pub fn cards(self) -> impl Iterator<Item = Card> {
        std::iter::repeat(Card::Skull)
            .take(self.num_skulls())
            .chain(std::iter::repeat(Card::Flower).take(self.num_flowers()))
    }

    #[must_use]
    pub fn remove_card(self, card: Card) -> Result<Option<Hand>, HandError> {
        let has_card = match card {
            Card::Skull if self.has_skull => true,
            Card::Flower if self.num_flowers() > 0 => true,
            _ => false,
        };
        if !has_card {
            Err(HandError::CardNotFound)
        } else if self.num_cards > 1 {
            match card {
                Card::Skull => Ok(Some(Self {
                    num_cards: self.num_cards - 1,
                    has_skull: false,
                })),
                Card::Flower => Ok(Some(Self {
                    num_cards: self.num_cards - 1,
                    has_skull: self.has_skull,
                })),
            }
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn add_card(self, card: Card) -> Result<Hand, HandError> {
        if self.num_cards >= 4 || (self.num_flowers() >= 3 && card == Card::Flower) {
            return Err(HandError::TooManyCards);
        }
        match card {
            Card::Skull if self.has_skull => Err(HandError::TooManyCards),
            Card::Skull => Ok(Self {
                num_cards: self.num_cards + 1,
                has_skull: true,
            }),
            Card::Flower => Ok(Self {
                num_cards: self.num_cards + 1,
                has_skull: self.has_skull,
            }),
        }
    }

    #[must_use]
    pub fn from_single_card(card: Card) -> Hand {
        Self {
            num_cards: 1,
            has_skull: match card {
                Card::Skull => true,
                Card::Flower => false,
            },
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum HandError {
    #[error("Too many cards in the hand")]
    TooManyCards,
    #[error("Card not found in the hand")]
    CardNotFound,
}

#[cfg(test)]
mod tests {
    use super::{Card, Hand, HandError};

    #[test]
    pub fn test_remove_cards_from_hand() {
        let h = Hand::new();

        assert_eq!(h.num_cards(), 4);
        assert_eq!(h.num_flowers(), 3);
        assert_eq!(h.num_skulls(), 1);

        let new_h = h.remove_card(Card::Flower).unwrap().unwrap();
        assert_eq!(new_h.num_cards(), 3);
        assert_eq!(new_h.num_flowers(), 2);
        assert_eq!(new_h.num_skulls(), 1);

        let new_h2 = new_h.remove_card(Card::Flower).unwrap().unwrap();
        assert_eq!(new_h2.num_cards(), 2);
        assert_eq!(new_h2.num_flowers(), 1);
        assert_eq!(new_h2.num_skulls(), 1);

        let new_h3 = new_h2.remove_card(Card::Flower).unwrap().unwrap();
        assert_eq!(new_h3.num_cards(), 1);
        assert_eq!(new_h3.num_flowers(), 0);
        assert_eq!(new_h3.num_skulls(), 1);

        assert_eq!(
            new_h3.remove_card(Card::Flower).unwrap_err(),
            HandError::CardNotFound
        );

        assert_eq!(new_h3.remove_card(Card::Skull).unwrap(), None);
    }

    #[test]
    pub fn test_add_cards_to_hand() {
        let h = Hand::from_single_card(Card::Skull);

        assert_eq!(
            h.add_card(Card::Skull).unwrap_err(),
            HandError::TooManyCards
        );

        assert_eq!(h.num_cards(), 1);
        assert_eq!(h.num_flowers(), 0);
        assert_eq!(h.num_skulls(), 1);

        let new_h = h.add_card(Card::Flower).unwrap();
        assert_eq!(new_h.num_cards(), 2);
        assert_eq!(new_h.num_flowers(), 1);
        assert_eq!(new_h.num_skulls(), 1);

        let new_h2 = new_h.add_card(Card::Flower).unwrap();
        assert_eq!(new_h2.num_cards(), 3);
        assert_eq!(new_h2.num_flowers(), 2);
        assert_eq!(new_h2.num_skulls(), 1);

        let new_h3 = new_h2.add_card(Card::Flower).unwrap();
        assert_eq!(new_h3.num_cards(), 4);
        assert_eq!(new_h3.num_flowers(), 3);
        assert_eq!(new_h3.num_skulls(), 1);

        assert_eq!(
            new_h3.add_card(Card::Flower).unwrap_err(),
            HandError::TooManyCards
        );
    }
}
