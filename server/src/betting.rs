use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct BettingState {
    pub pot: u32,
    pub to_call: u32,
    pub min_bet: u32,
    pub max_bet: u32,

    pub raises_used: u8,
    pub max_raises: u8,

    pub contributions: HashMap<Uuid, u32>, // per player, this round
    pub folded: HashSet<Uuid>,
}
