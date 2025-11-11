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

    pub fn get_exchange_rate(&self, asset_pair: &AssetPair) -> Option<Num> {
        self.exchange_rates.get(&asset_pair).map(Num::clone)    
    }
    
    pub fn set_exchange_rate(&mut self, asset_pair: AssetPair, rate: Num) {
        self.exchange_rates.insert(asset_pair, rate);
    }
}

pub struct OrderRequest {
    pub asset_pair: AssetPair,
    pub quantity_to_buy: Num,
    pub max_price: Num,
}

pub struct Order {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_quantity: Num,
    pub filled_percentage: Num,
}

pub struct Position {
    pub asset: String,
    pub quantity: Num,
}

#[derive(Hash, PartialEq, Eq)]
pub struct AssetPair {
    pub from_asset: String,
    pub to_asset: String,
}