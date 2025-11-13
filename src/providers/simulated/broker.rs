// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::Amount;
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

pub struct SimulatedBroker {
    orders: HashSet<Order>,
    orders_v2: HashMap<String, OrderV2>,
    exchange_rates: HashMap<AssetPair, Num>,
    balances: HashMap<String, Num>,
}

impl SimulatedBroker {
    pub fn new(starting_balances: HashMap<String, Num>) -> Self {
        Self {
            orders: HashSet::new(),
            orders_v2: HashMap::new(),
            exchange_rates: HashMap::new(),
            balances: starting_balances.clone(),
        }
    }

    // Only supports market orders, in this case they execute immediately since the exchange rate is determined in this method
    pub fn place_order_v2(&mut self, order_req: OrderRequestV2) -> Result<String> {
        let order_id = Uuid::new_v4().to_string();

        let asset_on_sale = &order_req.asset_pair.asset_on_sale;

        let exchange_rate = self
            .exchange_rates
            .get(&order_req.asset_pair)
            .ok_or(format_err!(
                "{} is not a valid asset pair",
                order_req.asset_pair
            ))?;

        let balance = self.balances.get(asset_on_sale).ok_or(format_err!(
            "Not enough {} balance to place the order",
            asset_on_sale
        ))?;

        let quantity: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => quantity,
            Amount::Notional { notional } => &(notional * exchange_rate),
        };

        let notional: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => &(quantity / exchange_rate),
            Amount::Notional { notional } => notional,
        };

        if balance < notional {
            return Err(format_err!(
                "Not enough {} balance to place the order",
                asset_on_sale
            ));
        }

        self.update_balance(asset_on_sale, -notional);
        self.update_balance(&order_req.asset_pair.asset_being_bought, quantity.clone());

        self.orders_v2.insert(
            order_id.clone(),
            OrderV2 {
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

    pub fn place_order(&mut self, order_req: OrderRequest) -> Result<String> {
        let order_id = Uuid::new_v4().to_string();

        let asset_on_sale = &order_req.asset_pair.asset_on_sale;
        let asset_being_bought = &order_req.asset_pair.asset_being_bought;

        let exchange_rate = self
            .exchange_rates
            .get(&order_req.asset_pair)
            .ok_or(format_err!(
                "{} is not a valid asset pair",
                order_req.asset_pair
            ))?;

        let balance = self
            .balances
            .get(asset_on_sale)
            .ok_or(format_err!("No available balance for {}", asset_on_sale))?;

        if *balance < &order_req.quantity_to_buy / &order_req.min_exchange_rate {
            return Err(format_err!(
                "Not enough {} balance to place the order",
                asset_on_sale
            ));
        }

        if exchange_rate >= &order_req.min_exchange_rate {
            self.orders.insert(Order {
                order_id: order_id.clone(),
                asset_pair: order_req.asset_pair.clone(),
                quantity: order_req.quantity_to_buy.clone(),
                max_price: exchange_rate.clone(),
                filled: true,
            });

            self.update_balance(asset_on_sale, -&order_req.quantity_to_buy / exchange_rate);
            self.update_balance(asset_being_bought, order_req.quantity_to_buy);
        } else {
            self.orders.insert(Order {
                order_id: order_id.clone(),
                asset_pair: order_req.asset_pair.clone(),
                quantity: order_req.quantity_to_buy.clone(),
                max_price: order_req.min_exchange_rate.clone(),
                filled: false,
            });

            self.update_balance(
                asset_on_sale,
                -&order_req.quantity_to_buy / &order_req.min_exchange_rate,
            );
        }

        Ok(order_id)
    }

    fn update_balance(&mut self, asset: &String, delta: Num) {
        let previous_balance = self
            .balances
            .get(asset)
            .map(|value| value.clone())
            .unwrap_or(Num::from(0));
        self.balances
            .insert(asset.clone(), previous_balance + delta);
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

pub struct OrderRequestV2 {
    pub asset_pair: AssetPair,
    pub amount: Amount,
}

pub struct OrderRequest {
    pub asset_pair: AssetPair,
    pub quantity_to_buy: Num,
    pub min_exchange_rate: Num,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Order {
    pub order_id: String,
    pub asset_pair: AssetPair,
    quantity: Num,
    max_price: Num,
    pub filled: bool,
}

#[derive(Hash, PartialEq, Eq)]
pub struct OrderV2 {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_amount: FilledAmount,
}

#[derive(Hash, PartialEq, Eq)]
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
