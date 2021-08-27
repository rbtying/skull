use serde::{Deserialize, Serialize};

pub mod bidding;
pub mod initialize;
pub mod placement;
pub mod selection;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum GameState {
    Initialize(initialize::Initialize),
    Placement(placement::Placement),
    Bidding(bidding::Bidding),
    Selection(selection::Selection),
}
