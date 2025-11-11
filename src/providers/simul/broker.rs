// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use std::collections::HashMap;

pub struct SimulBroker {
    orders: Vec<Order>,
    exchange_rates: HashMap<AssetPair, Num>,
    balance: HashMap<String, Num>
}

impl SimulBroker {
    pub fn new(starting_balance: HashMap<String, Num>) -> Self {
        Self {
            orders: vec![],
            exchange_rates: HashMap::new(),
            balance: starting_balance.clone()
        }
    }

    pub fn place_order(&mut self, order: Order) {
        self.orders.push(order);
    }

    pub fn set_exchange_rate(&mut self, asset_pair: AssetPair, rate: Num) {
        self.exchange_rates.insert(asset_pair, rate);
    }
}

pub struct Order {
    asset_pair: AssetPair,
    quantity_to_sell: Num,
    min_price: Num,
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