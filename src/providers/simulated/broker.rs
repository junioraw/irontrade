// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::Amount;
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub struct SimulatedBroker {
    orders: HashMap<String, Order>,
    exchange_rates: HashMap<AssetPair, Num>,
    balances: HashMap<String, Num>,
}

impl SimulatedBroker {
    pub fn new(starting_balances: HashMap<String, Num>) -> Self {
        Self {
            orders: HashMap::new(),
            exchange_rates: HashMap::new(),
            balances: starting_balances.clone(),
        }
    }

    // Only supports market orders, in this case they execute immediately since the exchange rate is determined in this method
    // TODO: Add support for limit orders
    pub fn place_order(&mut self, order_req: OrderRequest) -> Result<String> {
        let exchange_rate = &self.get_exchange_rate(&order_req.asset_pair)?;

        let quantity: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => quantity,
            Amount::Notional { notional } => &(notional * exchange_rate),
        };

        let notional: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => &(quantity / exchange_rate),
            Amount::Notional { notional } => notional,
        };

        let asset_on_sale = &order_req.asset_pair.asset_on_sale;
        let balance = &self.get_balance(asset_on_sale);

        if balance < notional {
            return Err(format_err!(
                "Not enough {} balance to place the order",
                asset_on_sale
            ));
        }

        self.update_balance(asset_on_sale, -notional);
        self.update_balance(&order_req.asset_pair.asset_being_bought, quantity.clone());

        let order_id = Uuid::new_v4().to_string();

        self.orders.insert(
            order_id.clone(),
            Order {
                order_id: order_id.clone(),
                asset_pair: order_req.asset_pair,
                filled_amount: FilledAmount {
                    quantity: quantity.clone(),
                    notional: notional.clone(),
                },
            },
        );

        Ok(order_id)
    }

    pub fn get_order(&self, order_id: &String) -> Result<Order> {
        self.orders
            .get(order_id)
            .map(Order::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_balance(&self, asset: &String) -> Num {
        self.balances
            .get(asset)
            .map(Num::clone)
            .unwrap_or(Num::from(0))
    }

    pub fn get_exchange_rate(&self, asset_pair: &AssetPair) -> Result<Num> {
        self.exchange_rates
            .get(&asset_pair)
            .map(Num::clone)
            .ok_or(format_err!("Asset pair {} can't be traded", asset_pair))
    }

    pub fn set_exchange_rate(&mut self, asset_pair: AssetPair, rate: Num) {
        self.exchange_rates.insert(asset_pair, rate);
    }

    fn update_balance(&mut self, asset: &String, delta: Num) {
        let previous_balance = self
            .balances
            .get(asset)
            .map(Num::clone)
            .unwrap_or(Num::from(0));
        self.balances
            .insert(asset.clone(), previous_balance + delta);
    }
}

pub struct OrderRequest {
    pub asset_pair: AssetPair,
    pub amount: Amount,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Order {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_amount: FilledAmount,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct FilledAmount {
    pub quantity: Num,
    pub notional: Num,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct AssetPair {
    pub asset_on_sale: String,
    pub asset_being_bought: String,
}

impl Display for AssetPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}/{}",
            self.asset_being_bought, self.asset_on_sale
        ))
    }
}
