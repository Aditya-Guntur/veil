use candid::{CandidType, Principal, Encode, Decode};
use serde::{Deserialize, Serialize};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use std::borrow::Cow;

pub type OrderId = u64;
pub type RoundId = u64;
pub type Timestamp = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Asset {
    BTC,
    ETH,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RoundState {
    Pending,      // Waiting to start
    Active,       // Accepting orders
    Revealing,    // Decrypting orders
    Clearing,     // Finding price
    Executing,    // Settling on-chain
    Completed,    // Done
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: OrderId,
    pub round_id: RoundId,
    pub owner: Principal,
    pub order_type: OrderType,
    pub asset: Asset,
    pub amount: u64,           // Amount in smallest unit (satoshis/wei)
    pub price_limit: u64,      // Price in USD cents (e.g., 67500 = $675.00)
    pub created_at: Timestamp,
    pub encrypted_payload: Vec<u8>,
    pub commitment_hash: String,  // Hash of unencrypted order for verification
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct OrderMatch {
    pub order_id: OrderId,
    pub filled: bool,
    pub fill_amount: u64,
    pub fill_price: u64,
    pub surplus: u64,  // Savings for buyer or extra earnings for seller
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ClearingResult {
    pub round_id: RoundId,
    pub clearing_price: u64,
    pub total_volume: u64,
    pub total_surplus: u64,
    pub matches: Vec<OrderMatch>,
    pub timestamp: Timestamp,
}

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct State {
    pub round_id: RoundId,
    pub round_state: RoundState,
    pub round_start_time: Timestamp,
    pub round_duration_ns: u64,  // 60 seconds = 60_000_000_000 nanoseconds
    pub next_order_id: OrderId,
    pub clearing_price_history: Vec<u64>,
}

impl Default for State {
    fn default() -> Self {
        State {
            round_id: 0,
            round_state: RoundState::Pending,
            round_start_time: 0,
            round_duration_ns: 60_000_000_000,  // 60 seconds
            next_order_id: 0,
            clearing_price_history: Vec::new(),
        }
    }
}

// Storable implementations for stable memory
impl Storable for Order {
    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Storable for ClearingResult {
    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

// User statistics
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct UserStats {
    pub user: Principal,
    pub total_orders: u64,
    pub filled_orders: u64,
    pub total_surplus: u64,
    pub rounds_participated: u64,
}

// Leaderboard entry
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub user: Principal,
    pub surplus: u64,
    pub fill_rate: u64,  // Percentage (0-100)
    pub rank: u64,
}