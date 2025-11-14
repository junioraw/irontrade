// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair};
use crate::api::request::MarketOrderRequest;
use anyhow::{format_err, Result};
use num_decimal::Num;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
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

    // Only supports market orders,
    // in this case they execute immediately since the exchange rate is determined in this method
    pub fn place_order(&mut self, order_req: MarketOrderRequest) -> Result<String> {
        let exchange_rate = &self.get_exchange_rate(&order_req.asset_pair)?;

        let quantity: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => quantity,
            Amount::Notional { notional } => &(notional / exchange_rate),
        };

        let notional: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => &(quantity * exchange_rate),
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
        self.exchange_rates.insert(asset_pair.clone(), rate.clone());
        self.exchange_rates
            .insert(asset_pair.inverse(), Num::from(1) / rate);
    }

    fn update_balance(&mut self, asset: &str, delta: Num) {
        let previous_balance = self
            .balances
            .get(asset)
            .map(Num::clone)
            .unwrap_or(Num::from(0));
        self.balances.insert(asset.into(), previous_balance + delta);
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct Order {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_amount: FilledAmount,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct FilledAmount {
    pub quantity: Num,
    pub notional: Num,
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::AssetPair;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn place_order_invalid_asset_pair() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from(1000));
        let mut broker = SimulatedBroker::new(balances);

        let err = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("AAPL/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "Asset pair AAPL/USD can't be traded");
    }

    #[test]
    fn place_order_no_balance() {
        let mut broker = SimulatedBroker::new(HashMap::new());
        broker.set_exchange_rate(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        let err = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD balance to place the order");

        broker.update_balance("USD", Num::from_str("13.09").unwrap());

        let err = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD balance to place the order");
    }

    #[test]
    fn place_order_updates_balances() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new(balances);
        broker.set_exchange_rate(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap();

        assert_eq!(broker.get_balance(&"USD".into()), Num::from(1));
        assert_eq!(broker.get_balance(&"GBP".into()), Num::from(10));
    }

    #[test]
    fn place_order_inverse_exchange_rate_updates_balances() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new(balances);
        broker.set_exchange_rate(
            AssetPair::from_str("USD/GBP").unwrap(),
            Num::from_str("0.8").unwrap(),
        );

        broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap();

        assert_eq!(
            broker.get_balance(&"USD".into()),
            Num::from_str("1.6").unwrap()
        );
        assert_eq!(broker.get_balance(&"GBP".into()), Num::from(10));
    }

    #[test]
    fn place_order_returns_valid_order_id() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new(balances);
        broker.set_exchange_rate(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        let order_id = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(order.order_id, order_id);
    }

    #[test]
    fn get_order_based_on_quantity_place_order() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new(balances);
        broker.set_exchange_rate(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        let order_id = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(
            order,
            Order {
                order_id,
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                filled_amount: FilledAmount {
                    quantity: Num::from(10),
                    notional: Num::from_str("13.1").unwrap()
                }
            }
        );
    }

    #[test]
    fn get_order_based_on_notional_place_order() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new(balances);
        broker.set_exchange_rate(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        let order_id = broker
            .place_order(MarketOrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from_str("6.55").unwrap(),
                },
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(
            order,
            Order {
                order_id,
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                filled_amount: FilledAmount {
                    quantity: Num::from(5),
                    notional: Num::from_str("6.55").unwrap()
                }
            }
        );
    }

    #[test]
    pub fn get_exchange_rate_inverse_set_exchange_rate() {
        let mut broker = SimulatedBroker::new(HashMap::new());
        broker.set_exchange_rate(
            AssetPair::from_str("USD/GBP").unwrap(),
            Num::from_str("0.8").unwrap(),
        );
        let exchange_rate = broker
            .get_exchange_rate(&AssetPair::from_str("GBP/USD").unwrap())
            .unwrap();
        assert_eq!(exchange_rate, Num::from_str("1.25").unwrap());
    }
}
