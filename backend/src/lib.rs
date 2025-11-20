use candid::Principal;
use ic_cdk::api::time;
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
use std::collections::HashMap;

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

const ORDERS_MEMORY_ID: MemoryId = MemoryId::new(1);
const RESULTS_MEMORY_ID: MemoryId = MemoryId::new(2);

thread_local! {
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
    
    // User stats - in-memory cache (could be moved to stable storage)
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
fn submit_order(
    order_type: OrderType,
    asset: Asset,
    amount: u64,
    price_limit: u64,
    encrypted_payload: Vec<u8>,
    commitment_hash: String,
) -> Result<OrderId, String> {
    let caller = ic_cdk::api::msg_caller();
    
    // Validate caller is not anonymous
    if caller == Principal::anonymous() {
        return Err("Anonymous users cannot submit orders".to_string());
    }
    
    // Check round state
    let current_state = STATE.with(|s| s.borrow().clone());
    
    if current_state.round_state != RoundState::Active {
        return Err(format!(
            "Cannot submit order: round is in {:?} state",
            current_state.round_state
        ));
    }
    
    // Validate inputs
    if amount == 0 {
        return Err("Amount must be greater than 0".to_string());
    }
    
    if price_limit == 0 {
        return Err("Price limit must be greater than 0".to_string());
    }
    
    if encrypted_payload.is_empty() {
        return Err("Encrypted payload cannot be empty".to_string());
    }
    
    if commitment_hash.is_empty() {
        return Err("Commitment hash cannot be empty".to_string());
    }
    
    // Create order
    let order_id = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let id = state.next_order_id;
        state.next_order_id += 1;
        id
    });
    
    let new_order = Order {
        id: order_id,
        round_id: current_state.round_id,
        owner: caller,
        order_type: order_type.clone(),
        asset: asset.clone(),
        amount,
        price_limit,
        created_at: time(),
        encrypted_payload,
        commitment_hash,
    };
    
    // Store order
    ORDERS.with(|orders| {
        orders.borrow_mut().insert(order_id, new_order);
    });
    
    ic_cdk::println!(
        "Order {} submitted: {:?} {} {:?} @ ${}",
        order_id,
        order_type,
        amount,
        asset,
        price_limit as f64 / 100.0
    );
    
    Ok(order_id)
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

#[update]
async fn get_encryption_public_key() -> Result<Vec<u8>, String> {
    encryption::get_encryption_public_key().await
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