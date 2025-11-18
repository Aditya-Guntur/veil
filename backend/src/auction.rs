use crate::types::{Order, OrderType, OrderMatch, ClearingResult};
use std::collections::BTreeMap;

pub fn find_clearing_price_and_match(
    orders: Vec<Order>,
    round_id: u64,
) -> Result<ClearingResult, String> {
    ic_cdk::println!("Starting clearing for round {} with {} orders", round_id, orders.len());
    
    // 1. Separate buy and sell orders
    let (mut buy_orders, mut sell_orders): (Vec<_>, Vec<_>) = orders
        .into_iter()
        .partition(|o| matches!(o.order_type, OrderType::Buy));
    
    if buy_orders.is_empty() || sell_orders.is_empty() {
        return Err(format!(
            "Cannot clear: {} buy orders, {} sell orders",
            buy_orders.len(),
            sell_orders.len()
        ));
    }
    
    // 2. Sort orders
    buy_orders.sort_by(|a, b| b.price_limit.cmp(&a.price_limit));  // Highest first
    sell_orders.sort_by(|a, b| a.price_limit.cmp(&b.price_limit)); // Lowest first
    
    // 3. Build cumulative supply/demand curves
    let mut demand_curve: BTreeMap<u64, u64> = BTreeMap::new();
    let mut cumulative_demand = 0u64;
    for order in &buy_orders {
        cumulative_demand += order.amount;
        demand_curve.insert(order.price_limit, cumulative_demand);
    }
    
    let mut supply_curve: BTreeMap<u64, u64> = BTreeMap::new();
    let mut cumulative_supply = 0u64;
    for order in &sell_orders {
        cumulative_supply += order.amount;
        supply_curve.insert(order.price_limit, cumulative_supply);
    }
    
    // 4. Find clearing price
    let mut all_prices: Vec<u64> = demand_curve
        .keys()
        .chain(supply_curve.keys())
        .copied()
        .collect();
    all_prices.sort();
    all_prices.dedup();
    
    let mut best_price = 0u64;
    let mut max_volume = 0u64;
    
    for &price in &all_prices {
        // Demand at this price or higher
        let demand = demand_curve
            .range(price..)
            .next_back()
            .map_or(0, |(_, &vol)| vol);
        
        // Supply at this price or lower
        let supply = supply_curve
            .range(..=price)
            .next_back()
            .map_or(0, |(_, &vol)| vol);
        
        let volume = demand.min(supply);
        
        if volume > max_volume {
            max_volume = volume;
            best_price = price;
        }
    }
    
    if max_volume == 0 {
        return Err("No clearing price found - orders don't overlap".to_string());
    }
    
    ic_cdk::println!("Found clearing price: ${}, volume: {}", best_price as f64 / 100.0, max_volume);
    
    // 5. Match orders at clearing price
    let mut matches = Vec::new();
    let mut remaining_volume = max_volume;
    let mut total_surplus = 0u64;
    
    // Match buy orders
    for order in buy_orders {
        if remaining_volume == 0 {
            // No more volume to match
            matches.push(OrderMatch {
                order_id: order.id,
                filled: false,
                fill_amount: 0,
                fill_price: 0,
                surplus: 0,
            });
            continue;
        }
        
        if order.price_limit >= best_price {
            let fill_amount = order.amount.min(remaining_volume);
            let surplus = (order.price_limit - best_price) * fill_amount;
            
            matches.push(OrderMatch {
                order_id: order.id,
                filled: true,
                fill_amount,
                fill_price: best_price,
                surplus,
            });
            
            total_surplus += surplus;
            remaining_volume -= fill_amount;
        } else {
            // Price too low, doesn't fill
            matches.push(OrderMatch {
                order_id: order.id,
                filled: false,
                fill_amount: 0,
                fill_price: 0,
                surplus: 0,
            });
        }
    }
    
    // Reset volume for sell side
    remaining_volume = max_volume;
    
    // Match sell orders
    for order in sell_orders {
        if remaining_volume == 0 {
            matches.push(OrderMatch {
                order_id: order.id,
                filled: false,
                fill_amount: 0,
                fill_price: 0,
                surplus: 0,
            });
            continue;
        }
        
        if order.price_limit <= best_price {
            let fill_amount = order.amount.min(remaining_volume);
            let surplus = (best_price - order.price_limit) * fill_amount;
            
            matches.push(OrderMatch {
                order_id: order.id,
                filled: true,
                fill_amount,
                fill_price: best_price,
                surplus,
            });
            
            total_surplus += surplus;
            remaining_volume -= fill_amount;
        } else {
            // Price too high, doesn't fill
            matches.push(OrderMatch {
                order_id: order.id,
                filled: false,
                fill_amount: 0,
                fill_price: 0,
                surplus: 0,
            });
        }
    }
    
    ic_cdk::println!("Matched {} orders with ${} total surplus", matches.len(), total_surplus as f64 / 100.0);
    
    Ok(ClearingResult {
        round_id,
        clearing_price: best_price,
        total_volume: max_volume,
        total_surplus,
        matches,
        timestamp: ic_cdk::api::time(),
    })
}