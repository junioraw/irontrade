// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair, Order, OrderSide, OrderStatus, OrderType};
use crate::api::request::OrderRequest;
use anyhow::{Result, format_err};
use num_decimal::Num;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub struct SimulatedBroker {
    currency: String,
    notional_assets: HashSet<String>,
    buying_power_balances: HashMap<String, Num>,
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
        let mut balances = HashMap::new();
        balances.insert(currency.clone(), Num::from(0));
        Self {
            currency,
            notional_assets,
            balances,
        }
    }

    pub fn set_balance(&mut self, balance: Num) -> &mut Self {
        self.balances.insert(self.currency.clone(), balance);
        self
    }

    pub fn add_notional_asset(&mut self, notional_asset: &str, balance: Option<Num>) -> &mut Self {
        self.notional_assets.insert(notional_asset.into());
        let effective_balance: Num;
        if let Some(balance) = balance {
            effective_balance = balance;
        } else {
            effective_balance = Num::from(0);
        }
        self.balances
            .insert(notional_asset.into(), effective_balance);
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
            orders: HashMap::new(),
            notional_per_unit: HashMap::new(),
            buying_power_balances: starting_balances.clone(),
            balances: starting_balances,
        })
    }

    pub fn place_order(&mut self, order_req: OrderRequest) -> Result<String> {
        let order_id = Uuid::new_v4().to_string();

        let type_ = match order_req.limit_price {
            None => OrderType::Market,
            Some(_) => OrderType::Limit,
        };

        let order = Order {
            order_id: order_id.clone(),
            asset_symbol: order_req.asset_pair.to_string(),
            amount: order_req.amount,
            limit_price: order_req.limit_price,
            filled_quantity: Num::from(0),
            average_fill_price: None,
            status: OrderStatus::New,
            type_,
            side: order_req.side,
        };

        self.queue_order(order.clone())?;

        if order.limit_price.is_some() {
            self.maybe_update_order(&order_id)?
        } else {
            self.fill_order_immediately(&order_id)?
        }

        Ok(order_id)
    }

    fn queue_order(&mut self, order: Order) -> Result<()> {
        let (asset, buying_power_needed) = self.get_asset_and_buying_power_needed(&order)?;
        let buying_power = self.get_buying_power(&asset)?;
        if buying_power < buying_power_needed {
            return Err(format_err!("Not enough {} buying power", asset));
        }
        self.orders.insert(order.order_id.clone(), order);
        Ok(())
    }

    fn get_asset_and_buying_power_needed(&self, order: &Order) -> Result<(String, Num)> {
        let asset_pair = &AssetPair::from_str(&order.asset_symbol)?;

        let (quantity, notional) =
            self.get_current_quantity_and_notional(&order.asset_symbol, &order.amount)?;

        let asset: &str;
        let buying_power_needed: Num;

        if order.side == OrderSide::Buy {
            asset = &asset_pair.notional_asset;
            buying_power_needed = order.limit_price.clone().unwrap_or_else(|| notional);
        } else {
            asset = &asset_pair.quantity_asset;
            buying_power_needed = quantity;
        }

        Ok((asset.to_string(), buying_power_needed))
    }

    fn maybe_update_order(&mut self, order_id: &String) -> Result<()> {
        let order = self.orders.get(order_id).unwrap().clone();
        let asset_pair = &AssetPair::from_str(&order.asset_symbol)?;
        let current_price = &self.get_notional_per_unit(asset_pair)?;
        let limit_price = &order.limit_price.clone().unwrap();

        if current_price == limit_price
            || ((order.side == OrderSide::Buy) == (current_price < limit_price))
        {
            self.fill_order_immediately(&order.order_id)?;
        }

        Ok(())
    }

    fn fill_order_immediately(&mut self, order_id: &String) -> Result<()> {
        let order = &self.orders.get(order_id).unwrap().clone();
        let (quantity, notional) =
            &self.get_current_quantity_and_notional(&order.asset_symbol, &order.amount)?;
        let asset_pair = &AssetPair::from_str(&order.asset_symbol)?;
        let notional_asset = &asset_pair.notional_asset;
        let quantity_asset = &asset_pair.quantity_asset;

        if order.side == OrderSide::Buy {
            self.update_balance(notional_asset, -notional)?;
            self.update_balance(quantity_asset, quantity.clone())?;
            if let Some(limit_price) = order.limit_price.clone() {
                self.update_buying_power(notional_asset, limit_price - notional)?;
            }
        } else {
            self.update_balance(notional_asset, notional.clone())?;
            self.update_balance(quantity_asset, -quantity)?;
        }

        self.orders.insert(
            order_id.clone(),
            Order {
                filled_quantity: quantity.clone(),
                average_fill_price: None,
                status: OrderStatus::Filled,
                ..order.clone()
            },
        );

        Ok(())
    }

    fn get_current_quantity_and_notional(
        &self,
        asset_symbol: &str,
        amount: &Amount,
    ) -> Result<(Num, Num)> {
        let asset_pair = &AssetPair::from_str(&asset_symbol)?;
        let notional_per_unit = &self.get_notional_per_unit(asset_pair)?;
        let quantity: Num = match amount {
            Amount::Quantity { quantity } => quantity.clone(),
            Amount::Notional { notional } => notional / notional_per_unit,
        };
        let notional: Num = match amount {
            Amount::Quantity { quantity } => quantity * notional_per_unit,
            Amount::Notional { notional } => notional.clone(),
        };
        Ok((quantity, notional))
    }

    pub fn get_orders(&self) -> Vec<Order> {
        self.orders.values().cloned().collect()
    }

    pub fn get_order(&self, order_id: &String) -> Result<Order> {
        self.orders
            .get(order_id)
            .map(Order::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_currency(&self) -> String {
        self.currency.clone()
    }

    pub fn get_buying_power(&self, asset: &str) -> Result<Num> {
        Self::get_asset_value(&self.buying_power_balances, asset)
    }

    pub fn get_balance(&self, asset: &str) -> Result<Num> {
        Self::get_asset_value(&self.balances, asset)
    }

    fn get_asset_value(values: &HashMap<String, Num>, asset: &str) -> Result<Num> {
        values
            .get(asset)
            .map(Num::clone)
            .ok_or_else(|| format_err!("Asset {} doesn't exist", asset))
    }

    pub fn get_notional_per_unit(&self, asset_pair: &AssetPair) -> Result<Num> {
        self.check_notional(asset_pair)?;
        self.notional_per_unit
            .get(&asset_pair)
            .map(Num::clone)
            .ok_or(format_err!(
                "{} does not have notional per unit",
                asset_pair
            ))
    }

    pub fn set_notional_per_unit(
        &mut self,
        asset_pair: AssetPair,
        notional_per_unit: Num,
    ) -> Result<()> {
        self.check_notional(&asset_pair)?;
        self.notional_per_unit
            .insert(asset_pair.clone(), notional_per_unit.clone());

        let order_ids: HashSet<String> = self.orders.keys().cloned().collect();
        for order_id in order_ids {
            self.maybe_update_order(&order_id)?
        }

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

    fn update_balance(&mut self, asset: &str, delta: Num) -> Result<()> {
        Self::update_value(&mut self.balances, asset, delta)
    }

    fn update_buying_power(&mut self, asset: &str, delta: Num) -> Result<()> {
        Self::update_value(&mut self.buying_power_balances, asset, delta)
    }

    fn update_value(values: &mut HashMap<String, Num>, asset: &str, delta: Num) -> Result<()> {
        let previous_balance = values
            .get(asset)
            .map(Num::clone)
            .ok_or_else(|| format_err!("Asset {} doesn't exist", asset))?;
        values.insert(asset.into(), previous_balance + delta);
        Ok(())
    }
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

        assert_eq!(err.to_string(), "AAPL/USD does not have notional per unit");
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
    fn place_order_updates_balances() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        let _ =
            broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?);

        broker.place_order(OrderRequest {
            asset_pair: AssetPair::from_str("GBP/USD")?,
            amount: Amount::Quantity {
                quantity: Num::from(10),
            },
            limit_price: None,
            side: OrderSide::Buy,
        })?;

        assert_eq!(broker.get_balance("USD")?, Num::from(1));
        assert_eq!(broker.get_balance("GBP")?, Num::from(10));

        Ok(())
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
    fn get_market_order() {
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
                limit_price: None,
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.31").unwrap()),
                status: OrderStatus::Filled,
                type_: OrderType::Market,
                side: OrderSide::Buy,
            }
        );
    }

    #[test]
    fn get_updated_limit_order() {
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
                limit_price: Some(Num::from_str("1.3").unwrap()),
                side: OrderSide::Buy,
            })
            .unwrap();

        let order = broker.get_order(&order_id).unwrap();
        assert_eq!(
            order,
            Order {
                order_id: order_id.clone(),
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.3").unwrap()),
                filled_quantity: Num::from(0),
                average_fill_price: None,
                status: OrderStatus::New,
                type_: OrderType::Limit,
                side: OrderSide::Buy,
            }
        );

        broker
            .set_notional_per_unit(
                AssetPair::from_str("GBP/USD").unwrap(),
                Num::from_str("1.29").unwrap(),
            )
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
                limit_price: Some(Num::from_str("1.3").unwrap()),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.29").unwrap()),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
                side: OrderSide::Buy,
            }
        );
    }

    #[test]
    fn get_filled_limit_order() {
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
                limit_price: Some(Num::from(2)),
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
                limit_price: Some(Num::from(2)),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.31").unwrap()),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
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
    fn build_no_balance() -> Result<()> {
        let broker = SimulatedBrokerBuilder::new("USD").build();
        assert_eq!(broker.get_balance("USD")?, Num::from(0));
        assert_eq!(broker.get_buying_power("USD")?, Num::from(0));
        Ok(())
    }

    #[test]
    fn build_negative_balance() -> Result<()> {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from(-10))
            .build();
        assert_eq!(broker.get_balance("USD")?, Num::from(-10));
        assert_eq!(broker.get_buying_power("USD")?, Num::from(-10));
        Ok(())
    }

    #[test]
    fn build_with_notional_assets() -> Result<()> {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .add_notional_asset("BTC", None)
            .add_notional_asset("USDT", Some(Num::from(-10)))
            .build();

        assert_eq!(
            broker.get_balance(&broker.get_currency())?,
            Num::from_str("14.1")?
        );
        assert_eq!(
            broker.get_buying_power(&broker.get_currency())?,
            Num::from_str("14.1")?
        );
        assert_eq!(broker.get_balance("USD")?, Num::from_str("14.1")?);
        assert_eq!(broker.get_buying_power("USD")?, Num::from_str("14.1")?);
        assert_eq!(broker.get_balance("USDT")?, Num::from(-10));
        assert_eq!(broker.get_buying_power("USDT")?, Num::from(-10));
        assert_eq!(broker.get_balance("BTC")?, Num::from(0));
        assert_eq!(broker.get_buying_power("BTC")?, Num::from(0));

        let err = broker.get_balance("GBP").unwrap_err();
        assert_eq!(err.to_string(), "Notional asset GBP doesn't exist");
        let err = broker.get_buying_power("GBP").unwrap_err();
        assert_eq!(err.to_string(), "Notional asset GBP doesn't exist");
        Ok(())
    }
}
