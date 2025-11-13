// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Error;
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub struct SimulatedBroker {
    orders: HashSet<Order>,
    exchange_rates: HashMap<AssetPair, Num>,
    balances: HashMap<String, Num>,
}

impl SimulatedBroker {
    pub fn new(starting_balances: HashMap<String, Num>) -> Self {
        Self {
            orders: HashSet::new(),
            exchange_rates: HashMap::new(),
            balances: starting_balances.clone(),
        }
    }

    pub fn place_order(&mut self, order: OrderRequest) -> Result<String> {
        let order_id = Uuid::new_v4().to_string();
        let exchange_rate = self.exchange_rates.get(&order.asset_pair);
        match exchange_rate {
            None => {
                return Err(format_err!(
                    "{} is not a valid asset pair",
                    order.asset_pair
                ));
            }
            Some(exchange_rate) => {
                let balance = self.balances.get(&order.asset_pair.from_asset);
                match balance {
                    None => {
                        return Err(format_err!(
                            "No available balance for {}",
                            &order.asset_pair.from_asset
                        ));
                    }
                    Some(balance) => {
                        if *balance < &order.quantity_to_buy * &order.max_price {
                            return Err(format_err!(
                                "Not enough {} balance to place the order",
                                order.asset_pair.from_asset
                            ));
                        }
                        if exchange_rate <= &order.max_price {
                            self.orders.insert(Order {
                                order_id: order_id.clone(),
                                asset_pair: order.asset_pair.clone(),
                                quantity: order.quantity_to_buy.clone(),
                                max_price: exchange_rate.clone(),
                                filled: true,
                            });
                            self.balances.insert(
                                order.asset_pair.from_asset.clone(),
                                balance - &order.quantity_to_buy * exchange_rate,
                            );
                            let previous_balance = self
                                .balances
                                .get(&order.asset_pair.to_asset)
                                .map(|value| value.clone())
                                .unwrap_or(Num::from(0));
                            self.balances.insert(
                                order.asset_pair.to_asset.clone(),
                                previous_balance + &order.quantity_to_buy,
                            );
                        } else {
                            self.orders.insert(Order {
                                order_id: order_id.clone(),
                                asset_pair: order.asset_pair.clone(),
                                quantity: order.quantity_to_buy.clone(),
                                max_price: order.max_price.clone(),
                                filled: false,
                            });
                            self.balances.insert(
                                order.asset_pair.from_asset.clone(),
                                balance - &order.quantity_to_buy * &order.max_price,
                            );
                        }
                    }
                }
            }
        }
        Ok(order_id)
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

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Order {
    pub order_id: String,
    pub asset_pair: AssetPair,
    quantity: Num,
    max_price: Num,
    pub filled: bool,
}

pub struct Position {
    pub asset: String,
    pub quantity: Num,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct AssetPair {
    pub from_asset: String,
    pub to_asset: String,
}

impl Display for AssetPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.to_asset, self.from_asset))
    }
}
