use crate::{Order, OrderType, ORDERS};
use std::collections::BTreeMap;

/// The result of a successful clearing.
#[derive(Debug)]
pub struct ClearingResult {
    pub clearing_price: u64,
    pub buy_volume: u64,
    pub sell_volume: u64,
}

/// Finds the uniform clearing price for the current set of orders.
pub fn find_clearing_price() -> Result<ClearingResult, String> {
    // 1. Separate buy and sell orders
    let (mut buy_orders, mut sell_orders) = ORDERS.with(|orders| {
        let orders = orders.borrow();
        let mut buys = Vec::new();
        let mut sells = Vec::new();
        for order in orders.values() {
            match order.order_type {
                OrderType::Buy => buys.push(order),
                OrderType::Sell => sells.push(order),
            }
        }
        (buys, sells)
    });

    // Handle edge case: zero orders
    if buy_orders.is_empty() || sell_orders.is_empty() {
        return Err("Not enough orders to determine a clearing price.".to_string());
    }

    // 2. Sort orders to build the order book
    // Buys: Sort high-to-low (highest price first)
    buy_orders.sort_by(|a, b| b.price_limit.cmp(&a.price_limit));
    // Sells: Sort low-to-high (lowest price first)
    sell_orders.sort_by(|a, b| a.price_limit.cmp(&b.price_limit));

    // 3. Create aggregate supply and demand curves
    // BTreeMap is used to aggregate volume at each price level
    let mut demand_curve: BTreeMap<u64, u64> = BTreeMap::new();
    let mut cumulative_demand = 0;
    for order in buy_orders {
        cumulative_demand += order.amount;
        demand_curve.insert(order.price_limit, cumulative_demand);
    }

    let mut supply_curve: BTreeMap<u64, u64> = BTreeMap::new();
    let mut cumulative_supply = 0;
    for order in sell_orders {
        cumulative_supply += order.amount;
        supply_curve.insert(order.price_limit, cumulative_supply);
    }

    // 4. Find the clearing price
    // We iterate through potential prices (from sorted sell prices)
    // and find where cumulative demand can meet cumulative supply.
    
    let mut best_price = 0;
    let mut max_volume = 0;

    // Iterate through all unique prices from both curves
    let mut all_prices: Vec<u64> = demand_curve.keys().chain(supply_curve.keys()).cloned().collect();
    all_prices.sort();
    all_prices.dedup();

    for price in all_prices {
        // Find the total demand at or above this price
        // We look for the lowest price in the demand curve that is >= current price
        let demand = demand_curve.range(price..).next_back().map_or(0, |(_, &vol)| vol);

        // Find the total supply at or below this price
        let supply = supply_curve.range(..=price).next_back().map_or(0, |(_, &vol)| vol);

        // The matched volume is the minimum of supply and demand
        let volume = demand.min(supply);

        if volume > max_volume {
            max_volume = volume;
            best_price = price;
        }
    }
    
    // 5. Handle edge case: no clearing price found
    if max_volume == 0 {
        return Err("No clearing price found. Orders do not cross.".to_string());
    }

    // Surplus Calculation (as described in the plan):
    // This is a conceptual explanation for now.
    // Total Buy Surplus = Sum of (buy_order.price_limit - clearing_price) * amount_filled
    // Total Sell Surplus = Sum of (clearing_price - sell_order.price_limit) * amount_filled
    // The `find_clearing_price` function's main job is just to find the price.
    // We will implement the actual surplus distribution later.

    Ok(ClearingResult {
        clearing_price: best_price,
        buy_volume: max_volume, // This is the total matched volume
        sell_volume: max_volume, // This is the total matched volume
    })
}