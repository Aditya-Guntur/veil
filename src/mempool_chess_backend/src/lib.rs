use candid::{CandidType, Decode, Encode, Principal};
use ic_cdk::api::time;
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::Deserialize;
use std::borrow::Cow;
use std::cell::RefCell;

mod auction;


// --- 1. TYPES & STRUCTS (Our Data Definitions) ---

// Using u64 for price and amount. Avoids floats in finance.
// E.g., for BTC, 1_000_000_000 units = 1 BTC (sats)
// E.g., for ETH, 1_000_000_000_000_000_000 units = 1 ETH (wei)

type OrderId = u64;
type Timestamp = u64; // Nanoseconds

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Asset {
    BTC,
    ETH,
    ICP, // Example
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Order {
    pub id: OrderId,
    pub owner: Principal,
    pub order_type: OrderType,
    pub asset: Asset,
    pub amount: u64,       // Amount of asset to buy/sell
    pub price_limit: u64,  // Max price for buys, Min price for sells
    pub created_at: Timestamp,
    // This is what vetKeys will encrypt!
    // For now, it's just a placeholder.
    // In Days 6-7, this will be the encrypted blob.
    pub encrypted_payload: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum RoundState {
    Pending, // Waiting to start
    Active,  // Accepting orders
    Revealing, // Decrypting and clearing
    Executing, // Executing trades
}

#[derive(CandidType, Deserialize, Clone)]
pub struct State {
    pub round_id: u64,
    pub round_state: RoundState,
    pub round_start_time: Timestamp,
    pub round_duration_ns: u64, // e.g., 60_000_000_000 for 60s
    pub next_order_id: OrderId,
}

impl Default for State {
    fn default() -> Self {
        State {
            round_id: 0,
            round_state: RoundState::Pending,
            round_start_time: 0,
            round_duration_ns: 60_000_000_000, // Default 60 seconds
            next_order_id: 0,
        }
    }
}

// --- 2. STABLE MEMORY (Our Database) ---

// Implement Storable trait for our Order struct
// THIS IS YOUR CURRENT, INCORRECT CODE
impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]>{
        Cow::Owned(Encode!(&self).unwrap())
    }
    fn into_bytes(self) -> Vec<u8>{
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

// Memory setup
type Memory = VirtualMemory<DefaultMemoryImpl>;
const STATE_MEMORY_ID: MemoryId = MemoryId::new(0);
const ORDERS_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    // Manages all our stable memory allocations
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Stores our single State struct
    static STATE: RefCell<State> = RefCell::new(State::default());

    // Stores all orders for the CURRENT round
    // Key: OrderId, Value: Order
    // This will be cleared after each round.
    static ORDERS: RefCell<StableBTreeMap<OrderId, Order, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ORDERS_MEMORY_ID))
        )
    );
}

// --- 3. API ENDPOINTS (Our Functions) ---

/// Submits a new order to the current active round.
/// This is the function the user's frontend will call.
#[update]
fn submit_order(
    order_type: OrderType,
    asset: Asset,
    amount: u64,
    price_limit: u64,
    encrypted_payload: Vec<u8>, // This will come from the client (Day 6-7)
) -> Result<OrderId, String> {
    
    // --- Basic State Check (Part of the state machine) ---
    let current_state = STATE.with(|s| s.borrow().clone());
    
    // For now, we'll manually start a round for testing.
    // Later, this check will be more robust.
    if current_state.round_state != RoundState::Active {
         // return Err("Order submission round is not active.".to_string());
         // For Day 1-2 testing, we'll just log this and proceed.
         ic_cdk::println!("Warning: Round is not active, but proceeding for test.");
    }
    
    // --- Create and save the order ---
    let order_id = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let id = state.next_order_id;
        state.next_order_id += 1;
        id
    });

    let new_order = Order {
        id: order_id,
        owner: ic_cdk::api::msg_caller(), // Get the Principal of the user
        order_type,
        asset,
        amount,
        price_limit,
        created_at: time(), // Get the current canister time
        encrypted_payload,
    };

    ORDERS.with(|orders| {
        orders.borrow_mut().insert(order_id, new_order);
    });

    ic_cdk::println!("Order submitted successfully: ID {}", order_id);
    Ok(order_id)
}

/// (Helper) Manually starts a new round.
/// We need this for testing Days 1-2.
#[update]
fn admin_start_round() -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_id += 1;
        state.round_state = RoundState::Active;
        state.round_start_time = time();
        
        // Clear orders from the previous round
        ORDERS.with(|orders| {
            let mut orders_mut = orders.borrow_mut();
            let keys: Vec<OrderId> = orders_mut.keys().collect();
            for k in keys {
                orders_mut.remove(&k);
            }
        });
        
        // Reset order counter for the new round
        state.next_order_id = 0;

        format!("Round {} started. Accepting orders.", state.round_id)
    })
}

/// (Helper) Gets the current round state.
#[query]
fn get_round_state() -> State {
    STATE.with(|s| s.borrow().clone())
}

/// (Helper) Gets the number of orders in the current round.
#[query]
fn get_order_count() -> u64 {
    ORDERS.with(|orders| orders.borrow().len())
}

/// (Helper) Runs the clearing algorithm on the current set of orders.
#[update]
fn admin_run_clearing() -> String {
    STATE.with(|s| {
        s.borrow_mut().round_state = RoundState::Revealing;
    });

    match auction::find_clearing_price() {
        Ok(result) => {
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Executing;
            });
            format!(
                "Clearing successful! Price: {}, Volume: {}",
                result.clearing_price, result.buy_volume
            )
        }
        Err(e) => {
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Pending; // Reset for next try
            });
            format!("Clearing failed: {}", e)
        }
    }
}


// --- 4. CANDID EXPORT ---
// This generates the .did file for our frontend to talk to
ic_cdk::export_candid!();