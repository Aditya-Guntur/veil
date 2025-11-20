use crate::types::*;
use crate::{ORDERS, RESULTS, USER_STATS, STATE};
use candid::Principal;
use std::collections::HashMap;

// ============================================================================
// USER QUERIES
// ============================================================================

/// Get all orders for a specific user
#[ic_cdk_macros::query]
pub fn get_user_orders(user: Principal) -> Vec<Order> {
    ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter_map(|entry| {
                let order = entry.value();
                if order.owner == user {
                    Some(order.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Get user's orders for current round only
#[ic_cdk_macros::query]
pub fn get_user_current_round_orders(user: Principal) -> Vec<Order> {
    let current_round = STATE.with(|s| s.borrow().round_id);
    
    ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter_map(|entry| {
                let order = entry.value();
                if order.owner == user && order.round_id == current_round {
                    Some(order.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Get user's statistics
#[ic_cdk_macros::query]
pub fn get_user_stats(user: Principal) -> Option<UserStats> {
    USER_STATS.with(|stats| {
        stats.borrow().get(&user).cloned()
    })
}

/// Get user's surplus for a specific round
#[ic_cdk_macros::query]
pub fn get_user_round_surplus(user: Principal, round_id: RoundId) -> u64 {
    // Get the clearing result
    let result = RESULTS.with(|results| {
        results.borrow().get(&round_id).map(|r| r.clone())
    });
    
    if let Some(result) = result {
        // Get user's orders for this round
        let user_order_ids: Vec<OrderId> = ORDERS.with(|orders| {
            orders
                .borrow()
                .iter()
                .filter_map(|entry| {
                    let (id, order) = (entry.key(), entry.value());
                    if order.owner == user && order.round_id == round_id {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect()
        });
        
        // Sum up surplus from matches
        result
            .matches
            .iter()
            .filter(|m| user_order_ids.contains(&m.order_id))
            .map(|m| m.surplus)
            .sum()
    } else {
        0
    }
}

// ============================================================================
// ROUND QUERIES
// ============================================================================

/// Get clearing result for a specific round
#[ic_cdk_macros::query]
pub fn get_round_result(round_id: RoundId) -> Option<ClearingResult> {
    RESULTS.with(|results| {
        results.borrow().get(&round_id).map(|r| r.clone())
    })
}

/// Get current round result (if available)
#[ic_cdk_macros::query]
pub fn get_current_round_result() -> Option<ClearingResult> {
    let current_round = STATE.with(|s| s.borrow().round_id);
    get_round_result(current_round)
}

/// Get all orders for a specific round
#[ic_cdk_macros::query]
pub fn get_round_orders(round_id: RoundId) -> Vec<Order> {
    ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter_map(|entry| {
                let order = entry.value();
                if order.round_id == round_id {
                    Some(order.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

/// Get clearing price history
#[ic_cdk_macros::query]
pub fn get_price_history() -> Vec<u64> {
    STATE.with(|s| s.borrow().clearing_price_history.clone())
}

/// Get last N clearing prices
#[ic_cdk_macros::query]
pub fn get_recent_prices(count: usize) -> Vec<u64> {
    STATE.with(|s| {
        let history = &s.borrow().clearing_price_history;
        let start = history.len().saturating_sub(count);
        history[start..].to_vec()
    })
}

// ============================================================================
// LEADERBOARD QUERIES
// ============================================================================

/// Get leaderboard for a specific round
#[ic_cdk_macros::query]
pub fn get_round_leaderboard(round_id: RoundId) -> Vec<LeaderboardEntry> {
    // Get clearing result
    let result = match RESULTS.with(|results| results.borrow().get(&round_id).map(|r| r.clone())) {
        Some(r) => r,
        None => return Vec::new(),
    };
    
    // Map order IDs to users
    let order_owners: HashMap<OrderId, Principal> = ORDERS.with(|orders| {
        orders
            .borrow()
            .iter()
            .filter_map(|entry| {
                let (id, order) = (entry.key(), entry.value());
                if order.round_id == round_id {
                    Some((*id, order.owner))
                } else {
                    None
                }
            })
            .collect()
    });
    
    // Calculate surplus per user
    let mut user_surplus: HashMap<Principal, u64> = HashMap::new();
    let mut user_orders: HashMap<Principal, (u64, u64)> = HashMap::new(); // (total, filled)
    
    for order_match in &result.matches {
        if let Some(user) = order_owners.get(&order_match.order_id) {
            *user_surplus.entry(*user).or_insert(0) += order_match.surplus;
            
            let (total, filled) = user_orders.entry(*user).or_insert((0, 0));
            *total += 1;
            if order_match.filled {
                *filled += 1;
            }
        }
    }
    
    // Build leaderboard
    let mut leaderboard: Vec<LeaderboardEntry> = user_surplus
        .into_iter()
        .map(|(user, surplus)| {
            let (total, filled) = user_orders.get(&user).unwrap_or(&(0, 0));
            let fill_rate = if *total > 0 {
                (*filled * 100) / *total
            } else {
                0
            };
            
            LeaderboardEntry {
                user,
                surplus,
                fill_rate,
                rank: 0, // Will be set below
            }
        })
        .collect();
    
    // Sort by surplus (descending)
    leaderboard.sort_by(|a, b| b.surplus.cmp(&a.surplus));
    
    // Assign ranks
    for (i, entry) in leaderboard.iter_mut().enumerate() {
        entry.rank = (i + 1) as u64;
    }
    
    leaderboard
}

/// Get global leaderboard (all-time)
#[ic_cdk_macros::query]
pub fn get_global_leaderboard() -> Vec<LeaderboardEntry> {
    USER_STATS.with(|stats| {
        let stats_map = stats.borrow();
        
        let mut leaderboard: Vec<LeaderboardEntry> = stats_map
            .values()
            .map(|user_stat| {
                let fill_rate = if user_stat.total_orders > 0 {
                    (user_stat.filled_orders * 100) / user_stat.total_orders
                } else {
                    0
                };
                
                LeaderboardEntry {
                    user: user_stat.user,
                    surplus: user_stat.total_surplus,
                    fill_rate,
                    rank: 0,
                }
            })
            .collect();
        
        // Sort by total surplus
        leaderboard.sort_by(|a, b| b.surplus.cmp(&a.surplus));
        
        // Assign ranks
        for (i, entry) in leaderboard.iter_mut().enumerate() {
            entry.rank = (i + 1) as u64;
        }
        
        leaderboard
    })
}

/// Get top N players by surplus
#[ic_cdk_macros::query]
pub fn get_top_players(count: usize) -> Vec<LeaderboardEntry> {
    let mut leaderboard = get_global_leaderboard();
    leaderboard.truncate(count);
    leaderboard
}

// ============================================================================
// ORDER BOOK QUERIES (for frontend display)
// ============================================================================

/// Get aggregated order book for current round (without revealing identities)
#[ic_cdk_macros::query]
pub fn get_order_book_summary() -> OrderBookSummary {
    let current_round = STATE.with(|s| s.borrow().round_id);
    
    let (buy_count, sell_count, total_buy_volume, total_sell_volume) = ORDERS.with(|orders| {
        let mut buy_count = 0u64;
        let mut sell_count = 0u64;
        let mut total_buy = 0u64;
        let mut total_sell = 0u64;
        
        for entry in orders.borrow().iter() {
            let order = entry.value();
            if order.round_id == current_round {
                match order.order_type {
                    OrderType::Buy => {
                        buy_count += 1;
                        total_buy += order.amount;
                    }
                    OrderType::Sell => {
                        sell_count += 1;
                        total_sell += order.amount;
                    }
                }
            }
        }
        
        (buy_count, sell_count, total_buy, total_sell)
    });
    
    OrderBookSummary {
        round_id: current_round,
        buy_orders: buy_count,
        sell_orders: sell_count,
        total_buy_volume,
        total_sell_volume,
    }
}

// Helper struct for order book summary
#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct OrderBookSummary {
    pub round_id: RoundId,
    pub buy_orders: u64,
    pub sell_orders: u64,
    pub total_buy_volume: u64,
    pub total_sell_volume: u64,
}

// ============================================================================
// STATISTICS QUERIES
// ============================================================================

/// Get overall platform statistics
#[ic_cdk_macros::query]
pub fn get_platform_stats() -> PlatformStats {
    let total_orders = ORDERS.with(|orders| orders.borrow().len());
    let total_rounds = STATE.with(|s| s.borrow().round_id);
    let total_users = USER_STATS.with(|stats| stats.borrow().len() as u64);
    
    let (total_volume, total_surplus) = RESULTS.with(|results| {
        let mut volume = 0u64;
        let mut surplus = 0u64;
        
        for entry in results.borrow().iter() {
            let result = entry.value();
            volume += result.total_volume;
            surplus += result.total_surplus;
        }
        
        (volume, surplus)
    });
    
    PlatformStats {
        total_orders,
        total_rounds,
        total_users,
        total_volume,
        total_surplus,
    }
}

// Helper struct for platform stats
#[derive(candid::CandidType, serde::Deserialize, Clone, Debug)]
pub struct PlatformStats {
    pub total_orders: u64,
    pub total_rounds: u64,
    pub total_users: u64,
    pub total_volume: u64,
    pub total_surplus: u64,
}