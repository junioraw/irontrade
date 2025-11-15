// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair};
use crate::api::request::MarketOrderRequest;
use crate::api::response::{Order, OrderStatus, OrderType};
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct SimulatedBroker {
    currency: String,
    notional_assets: HashSet<String>,
    orders: HashMap<String, FilledOrder>,
    exchange_rates: HashMap<AssetPair, Num>,
    balances: HashMap<String, Num>,
}

impl SimulatedBroker {
    pub fn new(currency: String, starting_balances: HashMap<String, Num>) -> Self {
        let mut notional_assets = HashSet::new();
        notional_assets.insert(currency.clone());
        Self::new_multiple_notional(currency, notional_assets, starting_balances).unwrap()
    }

    pub fn new_multiple_notional(
        currency: String,
        notional_assets: HashSet<String>,
        starting_balances: HashMap<String, Num>,
    ) -> Result<Self> {
        if !notional_assets.contains(&currency) {
            return Err(format_err!("Missing currency notional asset {}", currency));
        }
        Ok(Self {
            currency,
            notional_assets,
            orders: HashMap::new(),
            exchange_rates: HashMap::new(),
            balances: starting_balances.clone(),
        })
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

        let notional_asset = &order_req.asset_pair.notional_asset;
        let quantity_asset = &order_req.asset_pair.quantity_asset;

        let balance_err_asset;

        if notional >= &Num::from(0) {
            // buy order
            let balance = &self.get_balance(notional_asset);
            if balance < notional {
                balance_err_asset = Some(notional_asset);
            } else {
                balance_err_asset = None;
            }
        } else {
            // sell order
            let balance = self.get_balance(quantity_asset);
            if balance < -quantity {
                balance_err_asset = Some(quantity_asset);
            } else {
                balance_err_asset = None;
            }
        }

        if let Some(balance_err_asset) = balance_err_asset {
            return Err(format_err!(
                "Not enough {} balance to place the order",
                balance_err_asset
            ));
        }

        self.update_balance(notional_asset, -notional);
        self.update_balance(quantity_asset, quantity.clone());

        let order_id = Uuid::new_v4().to_string();

        self.orders.insert(
            order_id.clone(),
            FilledOrder {
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

    pub fn get_orders(&self) -> Vec<FilledOrder> {
        self.orders.values().cloned().collect()
    }

    pub fn get_order(&self, order_id: &String) -> Result<FilledOrder> {
        self.orders
            .get(order_id)
            .map(FilledOrder::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_balance(&self, asset: &String) -> Num {
        self.balances
            .get(asset)
            .map(Num::clone)
            .unwrap_or(Num::from(0))
    }

    pub fn get_exchange_rate(&self, asset_pair: &AssetPair) -> Result<Num> {
        self.check_notional(asset_pair)?;
        self.exchange_rates
            .get(&asset_pair)
            .map(Num::clone)
            .ok_or(format_err!("Asset pair {} can't be traded", asset_pair))
    }

    pub fn set_exchange_rate(&mut self, asset_pair: AssetPair, rate: Num) -> Result<()> {
        self.check_notional(&asset_pair)?;
        self.exchange_rates.insert(asset_pair.clone(), rate.clone());
        Ok(())
    }

    fn check_notional(&self, asset_pair: &AssetPair) -> Result<()> {
        if !self.notional_assets.contains(&asset_pair.notional_asset) {
            return Err(format_err!(
                "{} is not a valid notional asset",
                asset_pair.notional_asset,
            ));
        }
        Ok(())
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
pub struct FilledOrder {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_amount: FilledAmount,
}

impl From<FilledOrder> for Order {
    fn from(order: FilledOrder) -> Self {
        Self {
            order_id: order.order_id,
            asset_symbol: order.asset_pair.to_string(),
            amount: Amount::Quantity {
                quantity: order.filled_amount.quantity.clone(),
            },
            filled_quantity: order.filled_amount.quantity.clone(),
            average_fill_price: Some(order.filled_amount.quantity / order.filled_amount.notional),
            status: OrderStatus::Filled,
            type_: OrderType::Market,
        }
    }
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
        let mut broker = SimulatedBroker::new("USD".into(), balances);

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
        let mut broker = SimulatedBroker::new("USD".into(), HashMap::new());
        let _ = broker.set_exchange_rate(
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
        let mut broker = SimulatedBroker::new("USD".into(), balances);
        let _ = broker.set_exchange_rate(
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
    fn place_order_returns_valid_order_id() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new("USD".into(), balances);
        broker
            .set_exchange_rate(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

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
        let mut broker = SimulatedBroker::new("USD".into(), balances);
        broker
            .set_exchange_rate(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

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
            FilledOrder {
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
        let mut broker = SimulatedBroker::new("USD".into(), balances);
        broker
            .set_exchange_rate(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

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
            FilledOrder {
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
    fn set_exchange_rate_invalid_notional_asset() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new("USD".into(), balances);

        let err = broker
            .set_exchange_rate(
                AssetPair::from_str("GBP/USDT").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap_err();

        assert_eq!(err.to_string(), "USDT is not a valid notional asset");
    }

    #[test]
    fn set_exchange_rate_inverted_notional_asset() {
        let mut balances = HashMap::new();
        balances.insert("USD".into(), Num::from_str("14.1").unwrap());
        let mut broker = SimulatedBroker::new("USD".into(), balances);

        let err = broker
            .set_exchange_rate(
                AssetPair::from_str("USD/GBP").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap_err();

        assert_eq!(err.to_string(), "GBP is not a valid notional asset");
    }

    #[test]
    fn new_multiple_notional_without_currency() {
        let mut notional_assets = HashSet::new();
        notional_assets.insert("BTC".into());
        let err =
            SimulatedBroker::new_multiple_notional("USD".into(), notional_assets, HashMap::new())
                .unwrap_err();
        assert_eq!(err.to_string(), "Missing currency notional asset USD");
    }
}
