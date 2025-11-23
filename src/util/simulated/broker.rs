// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair, Order, OrderSide, OrderStatus, OrderType, OrderV1};
use crate::api::request::{OrderRequest, OrderRequestV1};
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct SimulatedBroker {
    currency: String,
    notional_assets: HashSet<String>,
    orders_v1: HashMap<String, BrokerOrder>,
    orders: HashMap<String, Order>,
    notional_per_unit: HashMap<AssetPair, Num>,
    balances: HashMap<String, Num>,
}

pub struct SimulatedBrokerBuilder {
    currency: String,
    notional_assets: HashSet<String>,
    balances: HashMap<String, Num>,
}

impl SimulatedBrokerBuilder {
    pub fn new(currency: &str) -> Self {
        let currency = currency.to_string();
        let mut notional_assets = HashSet::new();
        notional_assets.insert(currency.clone());
        Self {
            currency,
            notional_assets,
            balances: HashMap::new(),
        }
    }

    pub fn set_balance(&mut self, balance: Num) -> &mut Self {
        self.balances.insert(self.currency.clone(), balance);
        self
    }

    pub fn add_notional_asset(&mut self, notional_asset: &str, balance: Option<Num>) -> &mut Self {
        self.notional_assets.insert(notional_asset.into());
        if let Some(balance) = balance {
            self.balances.insert(notional_asset.into(), balance);
        }
        self
    }

    pub fn build(&self) -> SimulatedBroker {
        SimulatedBroker::new(
            &self.currency,
            self.notional_assets.clone(),
            self.balances.clone(),
        )
        .unwrap()
    }
}

impl SimulatedBroker {
    fn new(
        currency: &str,
        notional_assets: HashSet<String>,
        starting_balances: HashMap<String, Num>,
    ) -> Result<Self> {
        if !notional_assets.contains(currency) {
            return Err(format_err!("Missing currency notional asset {}", currency));
        }
        Ok(Self {
            currency: currency.into(),
            notional_assets,
            orders_v1: HashMap::new(),
            orders: HashMap::new(),
            notional_per_unit: HashMap::new(),
            balances: starting_balances.clone(),
        })
    }

    pub fn place_order(&mut self, order_req: OrderRequest) -> Result<String> {
        if order_req.limit_price.is_none() {
            return self.fill_order_immediately(
                &order_req.asset_pair,
                order_req.amount,
                OrderType::Market,
                order_req.side,
            );
        }

        let order_id = Uuid::new_v4().to_string();

        self.orders.insert(
            order_id.clone(),
            Order {
                order_id: order_id.clone(),
                asset_symbol: order_req.asset_pair.to_string(),
                amount: order_req.amount,
                filled_quantity: Num::from(0),
                average_fill_price: None,
                status: OrderStatus::New,
                type_: OrderType::Limit,
                side: order_req.side,
            },
        );

        Ok(order_id)
    }

    pub fn place_order_v1(&mut self, order_req: OrderRequestV1) -> Result<String> {
        let notional_per_unit = &self.get_notional_per_unit(&order_req.asset_pair)?;

        let quantity: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => quantity,
            Amount::Notional { notional } => &(notional / notional_per_unit),
        };

        let notional: &Num = match &order_req.amount {
            Amount::Quantity { quantity } => &(quantity * notional_per_unit),
            Amount::Notional { notional } => notional,
        };

        if order_req.limit_price.is_none() {
            // Market order
            return self.fill_v1_order_immediately(
                order_req.asset_pair,
                quantity,
                notional,
                OrderType::Market,
            );
        }

        let order_id = Uuid::new_v4().to_string();

        self.orders_v1.insert(
            order_id.clone(),
            BrokerOrder::PendingOrder(PendingOrder {
                order_id: order_id.clone(),
                asset_pair: order_req.asset_pair,
                amount: order_req.amount,
                limit_price: order_req.limit_price.unwrap(),
            }),
        );

        Ok(order_id)
    }

    fn fill_order_immediately(
        &mut self,
        asset_pair: &AssetPair,
        amount: Amount,
        order_type: OrderType,
        order_side: OrderSide,
    ) -> Result<String> {
        let notional_per_unit = &self.get_notional_per_unit(asset_pair)?;

        let notional_asset = &asset_pair.notional_asset;
        let quantity_asset = &asset_pair.quantity_asset;

        let balance_err_asset;

        let quantity: &Num = match &amount {
            Amount::Quantity { quantity } => quantity,
            Amount::Notional { notional } => &(notional / notional_per_unit),
        };

        let notional: &Num = match &amount {
            Amount::Quantity { quantity } => &(quantity * notional_per_unit),
            Amount::Notional { notional } => notional,
        };

        if order_side == OrderSide::Buy {
            let balance = &self.get_balance(notional_asset);
            if balance < notional {
                balance_err_asset = Some(notional_asset);
            } else {
                balance_err_asset = None;
            }
            self.update_balance(notional_asset, -notional);
            self.update_balance(quantity_asset, quantity.clone());
        } else {
            let balance = &self.get_balance(quantity_asset);
            if balance < quantity {
                balance_err_asset = Some(quantity_asset);
            } else {
                balance_err_asset = None;
            }
            self.update_balance(notional_asset, notional.clone());
            self.update_balance(quantity_asset, -quantity);
        }

        if let Some(balance_err_asset) = balance_err_asset {
            return Err(format_err!(
                "Not enough {} balance to place the order",
                balance_err_asset
            ));
        }

        let order_id = Uuid::new_v4().to_string();

        self.orders.insert(
            order_id.clone(),
            Order {
                order_id: order_id.clone(),
                asset_symbol: asset_pair.to_string(),
                amount: amount.clone(),
                filled_quantity: quantity.clone(),
                average_fill_price: Some(notional / quantity),
                status: OrderStatus::Filled,
                type_: order_type,
                side: order_side,
            },
        );

        Ok(order_id)
    }

    fn fill_v1_order_immediately(
        &mut self,
        asset_pair: AssetPair,
        quantity: &Num,
        notional: &Num,
        order_type: OrderType,
    ) -> Result<String> {
        let notional_asset = &asset_pair.notional_asset;
        let quantity_asset = &asset_pair.quantity_asset;

        let balance_err_asset;

        if quantity >= &Num::from(0) {
            // Buying
            let balance = &self.get_balance(notional_asset);
            if balance < notional {
                balance_err_asset = Some(notional_asset);
            } else {
                balance_err_asset = None;
            }
        } else {
            // Selling
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

        self.orders_v1.insert(
            order_id.clone(),
            BrokerOrder::FilledOrder(FilledOrder {
                order_id: order_id.clone(),
                asset_pair,
                filled_amount: FilledAmount {
                    quantity: quantity.clone(),
                    notional: notional.clone(),
                },
                order_type,
            }),
        );

        Ok(order_id)
    }

    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.values().cloned().collect()
    }

    pub fn get_orders_v1(&self) -> Vec<BrokerOrder> {
        self.orders_v1.values().cloned().collect()
    }

    pub fn get_order(&self, order_id: &String) -> Result<Order> {
        self.orders
            .get(order_id)
            .map(Order::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_order_v1(&self, order_id: &String) -> Result<BrokerOrder> {
        self.orders_v1
            .get(order_id)
            .map(BrokerOrder::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_currency(&self) -> String {
        self.currency.clone()
    }

    pub fn get_balance(&self, asset: &str) -> Num {
        self.balances
            .get(asset)
            .map(Num::clone)
            .unwrap_or(Num::from(0))
    }

    pub fn get_notional_per_unit(&self, asset_pair: &AssetPair) -> Result<Num> {
        self.check_notional(asset_pair)?;
        self.notional_per_unit
            .get(&asset_pair)
            .map(Num::clone)
            .ok_or(format_err!("{} is not a valid asset pair", asset_pair))
    }

    pub fn set_notional_per_unit(
        &mut self,
        asset_pair: AssetPair,
        notional_per_unit: Num,
    ) -> Result<()> {
        self.check_notional(&asset_pair)?;
        self.notional_per_unit
            .insert(asset_pair.clone(), notional_per_unit.clone());
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
pub enum BrokerOrder {
    FilledOrder(FilledOrder),
    PendingOrder(PendingOrder),
}

impl From<BrokerOrder> for OrderV1 {
    fn from(order: BrokerOrder) -> Self {
        match order {
            BrokerOrder::FilledOrder(order) => order.into(),
            BrokerOrder::PendingOrder(order) => order.into(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct FilledOrder {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub filled_amount: FilledAmount,
    pub order_type: OrderType,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct PendingOrder {
    pub order_id: String,
    pub asset_pair: AssetPair,
    pub amount: Amount,
    pub limit_price: Num,
}

impl From<FilledOrder> for OrderV1 {
    fn from(order: FilledOrder) -> Self {
        Self {
            order_id: order.order_id,
            asset_symbol: order.asset_pair.to_string(),
            amount: Amount::Quantity {
                quantity: order.filled_amount.quantity.clone(),
            },
            filled_quantity: order.filled_amount.quantity.clone(),
            average_fill_price: Some(order.filled_amount.notional / order.filled_amount.quantity),
            status: OrderStatus::Filled,
            type_: order.order_type,
        }
    }
}

impl From<PendingOrder> for OrderV1 {
    fn from(order: PendingOrder) -> Self {
        Self {
            order_id: order.order_id,
            asset_symbol: order.asset_pair.to_string(),
            amount: order.amount,
            filled_quantity: Num::from(0),
            average_fill_price: None,
            status: OrderStatus::New,
            type_: OrderType::Limit,
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
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        let err = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("AAPL/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "AAPL/USD is not a valid asset pair");
    }

    #[test]
    fn place_order_no_balance() {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

        let err = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD balance to place the order");
    }

    #[test]
    fn place_order_close_but_not_enough_balance() {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

        broker.update_balance("USD", Num::from_str("13.09").unwrap());

        let err = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD balance to place the order");
    }

    #[test]
    fn place_order_updates_balances() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        let _ = broker.set_notional_per_unit(
            AssetPair::from_str("GBP/USD").unwrap(),
            Num::from_str("1.31").unwrap(),
        );

        broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap();

        assert_eq!(broker.get_balance("USD"), Num::from(1));
        assert_eq!(broker.get_balance("GBP"), Num::from(10));
    }

    #[test]
    fn place_order_returns_valid_order_id() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

        let order_id = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(order.order_id, order_id);
    }

    #[test]
    fn get_order_based_on_quantity_place_order() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

        let order_id = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.31").unwrap()),
                status: OrderStatus::Filled,
                type_: OrderType::Market,
                side: OrderSide::Buy,
            }
        );
    }

    #[test]
    fn get_order_based_on_notional_place_order() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap();

        let order_id = broker
            .place_order(OrderRequest {
                asset_pair: AssetPair::from_str("GBP/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from_str("6.55").unwrap(),
                },
                limit_price: None,
                side: OrderSide::Buy,
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Notional {
                    notional: Num::from_str("6.55").unwrap(),
                },
                filled_quantity: Num::from(5),
                average_fill_price: Some(Num::from_str("1.31").unwrap()),
                status: OrderStatus::Filled,
                type_: OrderType::Market,
                side: OrderSide::Buy,
            }
        );
    }

    #[test]
    fn set_notional_per_unit_invalid_notional_asset() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        let err = broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USDT").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap_err();

        assert_eq!(err.to_string(), "USDT is not a valid notional asset");
    }

    #[test]
    fn set_notional_per_unit_inverted_notional_asset() {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .build();

        let err = broker
            .set_notional_per_unit(
                AssetPair::from_str("USD/GBP").unwrap(),
                Num::from_str("1.31").unwrap(),
            )
            .unwrap_err();

        assert_eq!(err.to_string(), "GBP is not a valid notional asset");
    }

    #[test]
    fn new_without_currency() {
        let mut notional_assets = HashSet::new();
        notional_assets.insert("BTC".into());
        let err = SimulatedBroker::new("USD", notional_assets, HashMap::new()).unwrap_err();
        assert_eq!(err.to_string(), "Missing currency notional asset USD");
    }

    #[test]
    fn build_no_balance() {
        let broker = SimulatedBrokerBuilder::new("USD").build();
        assert_eq!(broker.get_balance("USD"), Num::from(0))
    }

    #[test]
    fn build_negative_balance() {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from(-10))
            .build();
        assert_eq!(broker.get_balance("USD"), Num::from(-10))
    }

    #[test]
    fn build_with_notional_assets() {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1").unwrap())
            .add_notional_asset("BTC", None)
            .add_notional_asset("USDT", Some(Num::from(-10)))
            .build();

        assert_eq!(
            broker.get_balance(&broker.get_currency()),
            Num::from_str("14.1").unwrap()
        );
        assert_eq!(broker.get_balance("USD"), Num::from_str("14.1").unwrap());
        assert_eq!(broker.get_balance("USDT"), Num::from(-10));
        assert_eq!(broker.get_balance("BTC"), Num::from(0));
    }
}
