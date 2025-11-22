use candid::Principal;
use ic_cdk::api::{time, caller};
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::types::{ DemoUserBalance, ResultOrder};
use serde::{Deserialize, Serialize};

// ==============================
// Common Result Types
// ==============================

#[derive(candid::CandidType, serde::Serialize, serde::Deserialize, Debug)]
pub enum ResultBytes {
    Ok(Vec<u8>),
    Err(String),
}

// Import our modules
mod types;
mod auction;
mod encryption;
mod queries;
mod timers;

use types::*;
// Import the types needed for Candid export
use queries::{OrderBookSummary, PlatformStats};

// Memory setup
type Memory = VirtualMemory<DefaultMemoryImpl>;

type OrderId = u64;

const ORDERS_MEMORY_ID: MemoryId = MemoryId::new(1);
const RESULTS_MEMORY_ID: MemoryId = MemoryId::new(2);
const INITIAL_DEMO_BALANCE: u64 = 1_000_000_000; // 1.0 demo ckBTC in satoshis

const DEMO_USERS: [&str; 4] = [
        "trader1",
        "trader2",
        "trader3",
        "trader4",
];


thread_local! {
    static LAST_ORDER: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(vec![]);

    // test-storage
    pub static STORAGE: RefCell<Vec<(u64, Vec<u8>, String)>> = RefCell::new(vec![]);

    static VETKD_ID: RefCell<Option<Principal>> = RefCell::new(None);

    // balance setup for demo purposes
    static DEMO_BALANCES: std::cell::RefCell<HashMap<Principal, DemoUserBalance>> =
        std::cell::RefCell::new(HashMap::new());

    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static STATE: RefCell<State> = RefCell::new(State::default());

    // Orders storage - all orders across all rounds
    pub static ORDERS: RefCell<StableBTreeMap<OrderId, Order, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(ORDERS_MEMORY_ID))
        )
    );

    // Results storage - clearing results per round
    pub static RESULTS: RefCell<StableBTreeMap<RoundId, ClearingResult, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(RESULTS_MEMORY_ID))
        )
    );

    // User stats - in-memory cache
    static USER_STATS: RefCell<HashMap<Principal, UserStats>> = RefCell::new(HashMap::new());
}


// ============================================================================
// INITIALIZATION
// ============================================================================

#[init]
fn init() {
    ic_cdk::println!("Initializing Mempool Chess canister");
    
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_id = 0;
        state.round_state = RoundState::Pending;
        state.round_start_time = 0;
        state.round_duration_ns = 60_000_000_000; // 60 seconds
        state.next_order_id = 0;
        state.clearing_price_history = Vec::new();
    });
    
    ic_cdk::println!("Canister initialized successfully");
}

#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("Post-upgrade: Restoring state");
    // State is automatically restored from stable memory
    // Start the timer for automatic round progression
    timers::start_round_timer();
}

// ============================================================================
// ORDER SUBMISSION
// ============================================================================

#[update]
async fn submit_order(
    order_type: OrderType,
    asset: Asset,
    amount: u64,
    price_limit: u64,
    encrypted_payload: Vec<u8>,
    commitment_hash: String,
) -> ResultOrder {
    let caller = ic_cdk::caller();

    // 1) Basic round checks
    let state = STATE.with(|s| s.borrow().clone());
    if state.round_state != RoundState::Active {
        return ResultOrder::Err("Round is not active".to_string());
    }

    if amount == 0 {
        return ResultOrder::Err("Amount must be > 0".to_string());
    }

    // 2) Escrow: lock demo funds for this user
    if let Err(e) = lock_demo_funds(caller, &order_type, amount, price_limit) {
        return ResultOrder::Err(e);
    }

    // 3) Generate new OrderId
    let order_id = STATE.with(|s| {
        let mut st = s.borrow_mut();
        let id = st.next_order_id;
        st.next_order_id += 1;
        id
    });

    let now = ic_cdk::api::time();

    let order = Order {
        id: order_id,
        round_id: state.round_id,
        owner: caller,
        order_type: order_type.clone(),
        asset,
        amount,
        price_limit,
        created_at: now,
        encrypted_payload,
        commitment_hash,
    };

    // 4) Store order in ORDERS or ORDERS_BY_ROUND (depending on your structure)
    ORDERS.with(|orders| {
        orders.borrow_mut().insert(order_id, order);
    });

    ResultOrder::Ok(order_id)
}

// ============================================================================
// ROUND MANAGEMENT (Admin Functions)
// ============================================================================
#[update]
fn admin_start_round() -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        
        // Only start if pending
        if state.round_state != RoundState::Pending {
            return format!(
                "Cannot start round: currently in {:?} state",
                state.round_state
            );
        }
        
        state.round_id += 1;
        state.round_state = RoundState::Active;
        state.round_start_time = time();
        
        ic_cdk::println!(
            "Round {} started at {}. Duration: {}s",
            state.round_id,
            state.round_start_time,
            state.round_duration_ns / 1_000_000_000
        );
        
        format!(
            "Round {} started. Accepting orders for 60 seconds.",
            state.round_id
        )
    })
}

#[update]
async fn admin_run_clearing() -> String {
    let current_round = STATE.with(|s| s.borrow().round_id);
    
    ic_cdk::println!("Admin triggered clearing for round {}", current_round);
    
    // Change state to Revealing
    STATE.with(|s| {
        s.borrow_mut().round_state = RoundState::Revealing;
    });
    
    // Get all orders for current round
    let round_orders: Vec<Order> = ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter_map(|entry| {
                let order = entry.value();  // ✅ CORRECT
                if order.round_id == current_round {
                    Some(order.clone())
                } else {
                    None
                }
            })
            .collect()
    });
    
    if round_orders.is_empty() {
        STATE.with(|s| {
            s.borrow_mut().round_state = RoundState::Pending;
        });
        return format!("No orders to clear in round {}", current_round);
    }
    
    ic_cdk::println!("Decrypting {} orders...", round_orders.len());
    
    // Decrypt orders (in production, this would use vetKeys)
    let decrypted_orders = match encryption::decrypt_order_batch(round_orders).await {
        Ok(orders) => orders,
        Err(e) => {
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Pending;
            });
            return format!("Decryption failed: {}", e);
        }
    };
    
    ic_cdk::println!("Orders decrypted. Running auction...");
    
    // Change state to Clearing
    STATE.with(|s| {
        s.borrow_mut().round_state = RoundState::Clearing;
    });
    
    // Run auction
    match auction::find_clearing_price_and_match(decrypted_orders, current_round) {
        Ok(result) => {
            ic_cdk::println!(
                "Clearing successful! Price: ${}, Volume: {}, Surplus: ${}",
                result.clearing_price as f64 / 100.0,
                result.total_volume,
                result.total_surplus as f64 / 100.0
            );
            
            // Store result
            RESULTS.with(|results| {
                results.borrow_mut().insert(current_round, result.clone());
            });

            // Update price history
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                state.clearing_price_history.push(result.clearing_price);
                state.round_state = RoundState::Executing;
            });
            
            // Update user stats
            update_user_stats(&result);

            // DEMO ESCROW: apply clearing to demo balances
            if let Err(e) = apply_settlement_for_round(&result) {
                ic_cdk::println!("Settlement error: {}", e);
            }
            
            // In production, this would trigger cross-chain settlement
            // For now, we'll just mark as completed
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Completed;
            });
            
            format!(
                "Round {} cleared! Price: ${:.2}, Volume: {}, Surplus: ${:.2}",
                current_round,
                result.clearing_price as f64 / 100.0,
                result.total_volume,
                result.total_surplus as f64 / 100.0
            )
        }
        Err(e) => {
            ic_cdk::println!("Clearing failed: {}", e);
            STATE.with(|s| {
                s.borrow_mut().round_state = RoundState::Pending;
            });
            format!("Clearing failed: {}", e)
        }
    }
}

#[update]
fn admin_reset_round() -> String {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.round_state = RoundState::Pending;
        format!("Round {} reset to Pending state", state.round_id)
    })
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn update_user_stats(result: &ClearingResult) {
    USER_STATS.with(|stats| {
        let mut stats_map = stats.borrow_mut();
        
        // Get all orders for this round to know who participated
        let round_orders: HashMap<OrderId, Principal> = ORDERS.with(|orders| {
            orders
                .borrow()
                .iter()
                .filter_map(|entry| {
                    let order = entry.value();  // ✅ CORRECT
                    if order.round_id == result.round_id {
                        Some((*entry.key(), order.owner))
                    } else {
                        None
                    }
                })
                .collect()
        });
        
        // Update stats for each matched order
        for order_match in &result.matches {
            if let Some(user) = round_orders.get(&order_match.order_id) {
                let user_stat = stats_map.entry(*user).or_insert(UserStats {
                    user: *user,
                    total_orders: 0,
                    filled_orders: 0,
                    total_surplus: 0,
                    rounds_participated: 0,
                });
                
                user_stat.total_orders += 1;
                
                if order_match.filled {
                    user_stat.filled_orders += 1;
                    user_stat.total_surplus += order_match.surplus;
                }
            }
        }
        
        // Mark round participation for all unique users
        let unique_users: std::collections::HashSet<_> = round_orders.values().collect();
        for user in unique_users {
            if let Some(user_stat) = stats_map.get_mut(user) {
                user_stat.rounds_participated += 1;
            }
        }
    });
}

fn get_or_create_demo_balance(user: Principal) -> DemoUserBalance {
    DEMO_BALANCES.with(|b| {
        let mut map = b.borrow_mut();
        map.entry(user)
            .or_insert(DemoUserBalance {
                btc_free: 1_000_000_000,
                btc_locked: 0,
                usd_free: 10_000_000_000,
                usd_locked: 0,
            })
            .clone()
    })
}

fn set_demo_balance(user: Principal, balance: DemoUserBalance) {
    DEMO_BALANCES.with(|b| {
        b.borrow_mut().insert(user, balance);
    });
}

/// Lock funds when the user submits an order
/// For demo: we lock *amount* units, regardless of price
fn lock_demo_funds(user: Principal, order_type: &OrderType, amount: u64, price_limit: u64,) -> Result<(), String> {
    with_demo_balance_mut(&user, |bal| {
        match order_type {
            OrderType::Buy => {
                // For BUY: lock USD = amount * price_limit
                let required = amount
                    .checked_mul(price_limit)
                    .ok_or_else(|| "Overflow in required funds".to_string())?;

                if bal.usd_free < required {
                    return Err(format!(
                        "Insufficient USD balance: required {}, available {}",
                        required, bal.usd_free
                    ));
                }

                bal.usd_free -= required;
                bal.usd_locked += required;
            }
            OrderType::Sell => {
                // For SELL: lock BTC = amount
                if bal.btc_free < amount {
                    return Err(format!(
                        "Insufficient BTC balance: required {}, available {}",
                        amount, bal.btc_free
                    ));
                }

                bal.btc_free -= amount;
                bal.btc_locked += amount;
            }
        }

        Ok(())
    })
}

// Helper to get mutable balance for a user
fn with_demo_balance_mut<R>(user: &Principal, f: impl FnOnce(&mut DemoUserBalance) -> R) -> R {
    DEMO_BALANCES.with(|balances| {
        let mut map = balances.borrow_mut();
        let bal = map.entry(*user).or_insert_with(|| DemoUserBalance {
            btc_free: 1_000_000_000,
            btc_locked: 0,
            usd_free: 10_000_000_000,
            usd_locked: 0,
        });
        f(bal)
    })
}

fn apply_settlement_for_round(clearing: &ClearingResult) -> Result<(), String> {
    // Build a map from order_id -> order to avoid repeated lookups
    use std::collections::HashMap;

    let orders_by_id: HashMap<OrderId, Order> = ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .map(|entry| {
                let id = *entry.key();
                let order = entry.value().clone();
                (id, order)
            })
            .collect()
    });

    for m in &clearing.matches {
        let order = match orders_by_id.get(&m.order_id) {
            Some(o) => o,
            None => {
                ic_cdk::println!("Settlement: unknown order id {}", m.order_id);
                continue;
            }
        };

        let user = order.owner;
        let fill_amount = m.fill_amount;
        let clearing_price = clearing.clearing_price;

        match order.order_type {
            OrderType::Buy => {
                // reserved = amount * price_limit (at submission)
                let reserved = order
                    .amount
                    .checked_mul(order.price_limit)
                    .ok_or_else(|| "Overflow in reserved funds".to_string())?;

                let cost = fill_amount
                    .checked_mul(clearing_price)
                    .ok_or_else(|| "Overflow in settlement cost".to_string())?;

                if cost > reserved {
                    return Err(format!(
                        "Settlement invariant violated for BUY order {}: cost {} > reserved {}",
                        order.id, cost, reserved
                    ));
                }

                let refund = reserved - cost;

                with_demo_balance_mut(&user, |bal| {
                    // We expect usd_locked >= reserved, but be defensive
                    if bal.usd_locked < reserved {
                        ic_cdk::println!(
                            "Warning: usd_locked {} < reserved {} for user {:?}",
                            bal.usd_locked,
                            reserved,
                            user
                        );
                    } else {
                        bal.usd_locked -= reserved;
                    }

                    // Buyer pays 'cost' and gets BTC
                    bal.btc_free = bal.btc_free.saturating_add(fill_amount);
                    // Any leftover reserved funds are refunded as free USD
                    bal.usd_free = bal.usd_free.saturating_add(refund);
                });
            }

            OrderType::Sell => {
                // reserved BTC = amount (at submission)
                let reserved_btc = order.amount;
                let unsold_btc = reserved_btc.saturating_sub(fill_amount);

                let proceeds = fill_amount
                    .checked_mul(clearing_price)
                    .ok_or_else(|| "Overflow in seller proceeds".to_string())?;

                with_demo_balance_mut(&user, |bal| {
                    if bal.btc_locked < reserved_btc {
                        ic_cdk::println!(
                            "Warning: btc_locked {} < reserved_btc {} for user {:?}",
                            bal.btc_locked,
                            reserved_btc,
                            user
                        );
                    } else {
                        bal.btc_locked -= reserved_btc;
                    }

                    // Unsold BTC is returned
                    bal.btc_free = bal.btc_free.saturating_add(unsold_btc);
                    // Proceeds in USD are credited
                    bal.usd_free = bal.usd_free.saturating_add(proceeds);
                });
            }
        }
    }

    Ok(())
}

// ============================================================================
// BASIC QUERY FUNCTIONS
// ============================================================================

#[query]
fn get_round_state() -> State {
    STATE.with(|s| s.borrow().clone())
}

#[query]
fn get_order_count() -> u64 {
    ORDERS.with(|orders| orders.borrow().len())
}

#[query]
fn get_current_round_orders() -> u64 {
    let current_round = STATE.with(|s| s.borrow().round_id);
    
    ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter(|entry| entry.value().round_id == current_round)
            .count() as u64
    })
}

#[query]
fn get_time_remaining() -> u64 {
    STATE.with(|s| {
        let state = s.borrow();
        
        if state.round_state != RoundState::Active {
            return 0;
        }
        
        let current_time = time();
        let elapsed = current_time.saturating_sub(state.round_start_time);
        
        if elapsed >= state.round_duration_ns {
            0
        } else {
            state.round_duration_ns - elapsed
        }
    })
}

// ============================================================================
// ENCRYPTION PUBLIC KEY (for frontend)
// ============================================================================

#[ic_cdk_macros::query]
fn get_encryption_public_key() -> Vec<u8> {
    // Demo key for PocketIC
    let mut fake_key = vec![0u8; 32];
    fake_key[..4].copy_from_slice(b"DEMO");
    fake_key
}


// ============================================================================
// CANDID EXPORT
// ============================================================================

ic_cdk::export_candid!();

// ============================================================================
// GETRANDOM (Required for crypto libraries)
// ============================================================================

#[no_mangle]
fn getrandom(_buf: *mut u8, _len: usize) -> i32 {
    ic_cdk::trap("getrandom() not implemented. Use ic_cdk::api::management_canister::main::raw_rand()");
}

// ============================================================================
// Endpoints for frontend to get demo balances
// ============================================================================

#[ic_cdk_macros::query]
pub fn get_my_demo_balance() -> DemoUserBalance {
    let user = ic_cdk::caller();
    get_or_create_demo_balance(user)
}

#[ic_cdk_macros::query]
pub fn get_demo_balance_of(user: Principal) -> DemoUserBalance {
    get_or_create_demo_balance(user)
}

// ============================================================================
// test-only methods
// ============================================================================
#[ic_cdk_macros::update]
fn pocketic_submit_order(
    _round_id: u64,
    encrypted_payload: Vec<u8>,
    _commitment_hash: String
) -> Result<u64, String> {
    LAST_ORDER.with(|o| *o.borrow_mut() = encrypted_payload);
    Ok(1)
}


#[ic_cdk_macros::query]
fn pocketic_get_order_ciphertext() -> Vec<u8> {
    LAST_ORDER.with(|o| o.borrow().clone())
}

#[ic_cdk_macros::update]
pub fn set_vetkd_canister(id: Principal) {
    VETKD_ID.with(|v| *v.borrow_mut() = Some(id));
}

pub fn vetkeys_engine_canister_id() -> Principal {
    VETKD_ID.with(|v| v.borrow().expect("VETKD canister not set"))
}
