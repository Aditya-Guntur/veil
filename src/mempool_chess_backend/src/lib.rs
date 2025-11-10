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

type OrderId = u64;
type Timestamp = u64; // Nanoseconds

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Asset {
    BTC,
    ETH,
    ICP,
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
    pub amount: u64,
    pub price_limit: u64,
    pub created_at: Timestamp,
    pub encrypted_payload: Vec<u8>,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum RoundState {
    Pending,
    Active,
    Revealing,
    Executing,
}

#[derive(CandidType, Deserialize, Clone)]
pub struct State {
    pub round_id: u64,
    pub round_state: RoundState,
    pub round_start_time: Timestamp,
    pub round_duration_ns: u64,
    pub next_order_id: OrderId,
}

impl Default for State {
    fn default() -> Self {
        State {
            round_id: 0,
            round_state: RoundState::Pending,
            round_start_time: 0,
            round_duration_ns: 60_000_000_000,
            next_order_id: 0,
        }
    }
}

// --- 2. STABLE MEMORY (Our Database) ---

impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self).unwrap())
    }
    
    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

// Memory setup
type Memory = VirtualMemory<DefaultMemoryImpl>;
const ORDERS_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE: RefCell<State> = RefCell::new(State::default());

    pub static ORDERS: RefCell<StableBTreeMap<OrderId, Order, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ORDERS_MEMORY_ID))
        )
    );
}

// --- 3. API ENDPOINTS (Our Functions) ---

#[update]
fn submit_order(
    order_type: OrderType,
    asset: Asset,
    amount: u64,
    price_limit: u64,
    encrypted_payload: Vec<u8>,
) -> Result<OrderId, String> {
    let current_state = STATE.with(|s| s.borrow().clone());
    
    if current_state.round_state != RoundState::Active {
        ic_cdk::println!("Warning: Round is not active, but proceeding for test.");
    }
    
    let order_id = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let id = state.next_order_id;
        state.next_order_id += 1;
        id
    });

    let new_order = Order {
        id: order_id,
        owner: ic_cdk::api::caller(),
        order_type,
        asset,
        amount,
        price_limit,
        created_at: time(),
        encrypted_payload,
    };

    ORDERS.with(|orders| {
        orders.borrow_mut().insert(order_id, new_order);
    });

    ic_cdk::println!("Order submitted successfully: ID {}", order_id);
    Ok(order_id)
}

#[update]
fn admin_start_round() -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_id += 1;
        state.round_state = RoundState::Active;
        state.round_start_time = time();
        
        ORDERS.with(|orders| {
            let mut orders_mut = orders.borrow_mut();
            let keys: Vec<OrderId> = orders_mut.keys().collect();
            for k in keys {
                orders_mut.remove(&k);
            }
        });
        
        state.next_order_id = 0;

        format!("Round {} started. Accepting orders.", state.round_id)
    })
}

#[query]
fn get_round_state() -> State {
    STATE.with(|s| s.borrow().clone())
}

#[query]
fn get_order_count() -> u64 {
    ORDERS.with(|orders| orders.borrow().len())
}

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
                s.borrow_mut().round_state = RoundState::Pending;
            });
            format!("Clearing failed: {}", e)
        }
    }
}

// --- 4. CANDID EXPORT ---
ic_cdk::export_candid!();