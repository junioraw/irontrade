// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use std::collections::HashMap;
use anyhow::Result;

pub struct SimulatedBroker {
    order_requests: Vec<OrderRequest>,
    orders: Vec<Order>,
    exchange_rates: HashMap<AssetPair, Num>,
    balance: HashMap<String, Num>
}

impl SimulatedBroker {
    pub fn new(starting_balance: HashMap<String, Num>) -> Self {
        Self {
            order_requests: vec![],
            orders: vec![],
            exchange_rates: HashMap::new(),
            balance: starting_balance.clone()
        }
    }

    pub fn place_order(&mut self, order: OrderRequest) -> Result<String> {
        self.order_requests.push(order);
        todo!()
    }

    pub fn get_order(self, order_id: String) -> Result<Order> {
        todo!()
    }

    pub fn get_positions() -> HashMap<String, Num> {
        todo!()
    }

    pub fn set_exchange_rate(&mut self, asset_pair: AssetPair, rate: Num) {
        self.exchange_rates.insert(asset_pair, rate);
    }
}

pub struct OrderRequest {
    asset_pair: AssetPair,
    quantity_to_sell: Num,
    min_price: Num,
}

pub struct Order {
    order_id: String,
    asset_pair: AssetPair,
    filled_quantity: Num,
    filled_percentage: Num,
}

pub struct Position {
    asset: String,
    quantity: Num,
}

#[derive(Hash, PartialEq, Eq)]
pub struct AssetPair {
    from_asset: String,
    to_asset: String,
}