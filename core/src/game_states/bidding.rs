use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game_states::selection::Selection;
use crate::types::{Card, Hand, PlayerID, Players};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum Bid {
    Pass,
    Amount(u8),
}

/// In the bidding phase, players (in order) make bids until:
/// 1. all players have a defined bid
/// 2. exactly one player has a `Bid::Amount`
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Bidding {
    players: Players,
    hands: HashMap<PlayerID, Hand>,
    cards: HashMap<PlayerID, Vec<Card>>,
    /// A player has either no bid, an explicit pass, or a bid with a particular amount. This map
    /// should never be empty, since we start the bidding phase when someone makes a bid.
    bids: HashMap<PlayerID, Bid>,
    current_player: PlayerID,
}

impl Bidding {
    #[must_use]
    pub fn new(
        players: Players,
        hands: HashMap<PlayerID, Hand>,
        cards: HashMap<PlayerID, Vec<Card>>,
        first_bid: (PlayerID, u8),
    ) -> Result<Self, BiddingError> {
        if first_bid.1 as usize > cards.values().map(|c| c.len()).max().unwrap_or(0) {
            return Err(BiddingError::BidTooHigh);
        }
        if first_bid.1 == 0 {
            return Err(BiddingError::BidTooLow);
        }

        let mut bids = HashMap::new();
        bids.insert(first_bid.0, Bid::Amount(first_bid.1));

        let next_player = players
            .next_player(first_bid.0)
            .map(|p| p.player_id)
            .ok_or(BiddingError::PlayerDoesntExist)?;
        if players.player_ids().len() < 2 {
            return Err(BiddingError::InsufficientPlayers);
        }

        Ok(Self {
            current_player: next_player,
            players,
            hands,
            cards,
            bids,
        })
    }

    #[must_use]
    pub fn make_bid(&self, player_id: PlayerID, bid: Bid) -> Result<BiddingResult, BiddingError> {
        let existing_bid = self.bids.get(&player_id).copied();
        let offset = self
            .players
            .player_ids()
            .iter()
            .position(|p| *p == player_id)
            .ok_or(BiddingError::PlayerDoesntExist)?;

        let min_bid = self
            .bids
            .values()
            .flat_map(|b| match b {
                Bid::Amount(v) => Some(v),
                Bid::Pass => None,
            })
            .max()
            .copied()
            .unwrap_or(0);
        let max_bid = self.cards.values().map(|c| c.len()).sum();

        let res = match (existing_bid, bid) {
            (Some(Bid::Pass), Bid::Pass) | (Some(Bid::Pass), Bid::Amount(_)) => {
                Err(BiddingError::AlreadyPassed)
            }
            (None, Bid::Amount(n)) | (Some(Bid::Amount(_)), Bid::Amount(n))
                if n <= min_bid || n as usize > max_bid =>
            {
                Err(BiddingError::BidTooLow)
            }
            (None, Bid::Pass)
            | (None, Bid::Amount(_))
            | (Some(Bid::Amount(_)), Bid::Pass)
            | (Some(Bid::Amount(_)), Bid::Amount(_)) => Ok(()),
        };

        let new_bidding = res.map(|()| {
            let mut new_bids = self.bids.clone();
            new_bids.insert(player_id, bid);

            let next_player = {
                // Find the next player who has never passed.
                let mut next = player_id;
                let num_players = self.players.player_ids().len();
                for i in 0..num_players {
                    let p = self.players.player_ids()[(i + offset) % num_players];
                    if new_bids.get(&p).copied() != Some(Bid::Pass) {
                        next = p;
                        break;
                    }
                }
                next
            };

            Self {
                players: self.players.clone(),
                hands: self.hands.clone(),
                cards: self.cards.clone(),
                bids: new_bids,
                current_player: next_player,
            }
        })?;

        if let Ok(selection) = new_bidding.finish_bidding() {
            Ok(BiddingResult::StartSelection(selection))
        } else {
            Ok(BiddingResult::KeepBidding(new_bidding))
        }
    }

    #[must_use]
    fn finish_bidding(&self) -> Result<Selection, BiddingError> {
        let num_passes = self
            .bids
            .values()
            .filter(|v| match v {
                Bid::Amount(_) => false,
                Bid::Pass => true,
            })
            .count();
        let mut iter = self.bids.iter().flat_map(|(k, v)| match v {
            Bid::Amount(amt) => Some((k, amt)),
            Bid::Pass => None,
        });
        let (selector, goal) = iter.next().ok_or(BiddingError::BiddingIncomplete)?;
        // We advance to selection if everyone other than the current selector has passed.
        if !iter.next().is_none() && num_passes == self.players.player_ids().len() - 1 {
            let selection = Selection::new(
                *selector,
                *goal,
                self.players.clone(),
                self.cards.clone(),
                self.hands.clone(),
            )
            .map_err(|()| BiddingError::BidTooHigh)?;
            Ok(selection)
        } else {
            Err(BiddingError::BiddingIncomplete)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BiddingResult {
    KeepBidding(Bidding),
    StartSelection(Selection),
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum BiddingError {
    #[error("That player doesn't exist")]
    PlayerDoesntExist,
    #[error("Insufficent number of players")]
    InsufficientPlayers,
    #[error("Player has already passed")]
    AlreadyPassed,
    #[error("Bid not higher than existing bid")]
    BidTooLow,
    #[error("Bid higher than acheivable")]
    BidTooHigh,
    #[error("All other players must pass")]
    BiddingIncomplete,
}
