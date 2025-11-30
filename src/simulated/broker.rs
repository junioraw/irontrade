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
        let buying_power = self.get_buying_power(&asset);
        if buying_power < buying_power_needed {
            return Err(format_err!("Not enough {} buying power", asset));
        }
        self.update_buying_power(&asset, -buying_power_needed);
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
            if let Some(limit_price) = &order.limit_price {
                buying_power_needed = limit_price * quantity;
            } else {
                buying_power_needed = notional;
            }
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
            self.update_balance(notional_asset, -notional);
            self.update_balance(quantity_asset, quantity.clone());
            self.update_buying_power(quantity_asset, quantity.clone());
            if let Some(limit_price) = order.limit_price.clone() {
                self.update_buying_power(notional_asset, limit_price * quantity - notional);
            }
        } else {
            self.update_balance(notional_asset, notional.clone());
            self.update_buying_power(notional_asset, notional.clone());
            self.update_balance(quantity_asset, -quantity);
        }

        self.orders.insert(
            order_id.clone(),
            Order {
                filled_quantity: quantity.clone(),
                average_fill_price: Some(notional / quantity),
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

    pub fn get_order(&self, order_id: &str) -> Result<Order> {
        self.orders
            .get(order_id)
            .map(Order::clone)
            .ok_or(format_err!("Order with id {} doesn't exist", order_id))
    }

    pub fn get_currency(&self) -> String {
        self.currency.clone()
    }

    pub fn get_buying_power(&self, asset: &str) -> Num {
        Self::get_asset_value(&self.buying_power_balances, asset)
    }

    pub fn get_balance(&self, asset: &str) -> Num {
        Self::get_asset_value(&self.balances, asset)
    }

    fn get_asset_value(values: &HashMap<String, Num>, asset: &str) -> Num {
        values.get(asset).map(Num::clone).unwrap_or(Num::from(0))
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

    fn update_balance(&mut self, asset: &str, delta: Num) {
        Self::update_value(&mut self.balances, asset, delta)
    }

    fn update_buying_power(&mut self, asset: &str, delta: Num) {
        Self::update_value(&mut self.buying_power_balances, asset, delta)
    }

    fn update_value(values: &mut HashMap<String, Num>, asset: &str, delta: Num) {
        let previous_balance = values.get(asset).map(Num::clone).unwrap_or(Num::from(0));
        values.insert(asset.into(), previous_balance + delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::AssetPair;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn place_order_invalid_asset_pair() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("AAPL/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let err = broker.place_order(order_request).unwrap_err();

        assert_eq!(err.to_string(), "AAPL/USD does not have notional per unit");

        Ok(())
    }

    #[test]
    fn place_order_no_balance() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let err = broker.place_order(order_request).unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD buying power");
        Ok(())
    }

    #[test]
    fn place_order_close_but_not_enough_balance() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        broker.update_balance("USD", Num::from_str("13.09")?);

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let err = broker.place_order(order_request).unwrap_err();

        assert_eq!(err.to_string(), "Not enough USD buying power");

        Ok(())
    }

    #[test]
    fn place_order_updates_balances() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        let _ =
            broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?);

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        broker.place_order(order_request)?;

        assert_eq!(broker.get_balance("USD"), Num::from(1));
        assert_eq!(broker.get_buying_power("USD"), Num::from(1));
        assert_eq!(broker.get_balance("GBP"), Num::from(10));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(10));

        Ok(())
    }

    #[test]
    fn place_order_returns_valid_order_id() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let order_id = broker.place_order(order_request)?;
        let order = broker.get_order(&order_id)?;

        assert_eq!(order.order_id, order_id);

        Ok(())
    }

    #[test]
    fn get_market_buy_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.32")?)?;

        let order_request = OrderRequest::create_market_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let order_id = broker.place_order(order_request)?;

        let actual_order = broker.get_order(&order_id)?;

        let expected_order = Order {
            order_id,
            asset_symbol: "GBP/USD".into(),
            amount: Amount::Quantity {
                quantity: Num::from(10),
            },
            limit_price: None,
            filled_quantity: Num::from(10),
            average_fill_price: Some(Num::from_str("1.32")?),
            status: OrderStatus::Filled,
            type_: OrderType::Market,
            side: OrderSide::Buy,
        };

        assert_eq!(actual_order, expected_order);

        assert_eq!(broker.get_balance("USD"), Num::from_str("0.9")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("0.9")?);
        assert_eq!(broker.get_balance("GBP"), Num::from(10));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(10));

        Ok(())
    }

    #[test]
    fn get_market_sell_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        broker.update_balance("GBP", Num::from(11));
        broker.update_buying_power("GBP", Num::from(11));

        let order_request = OrderRequest::create_market_sell(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
        );

        let order_id = broker.place_order(order_request)?;

        let actual_order = broker.get_order(&order_id)?;

        let expected_order = Order {
            order_id,
            asset_symbol: "GBP/USD".into(),
            amount: Amount::Quantity {
                quantity: Num::from(10),
            },
            limit_price: None,
            filled_quantity: Num::from(10),
            average_fill_price: Some(Num::from_str("1.31")?),
            status: OrderStatus::Filled,
            type_: OrderType::Market,
            side: OrderSide::Sell,
        };

        assert_eq!(actual_order, expected_order);

        assert_eq!(broker.get_balance("USD"), Num::from_str("13.1")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("13.1")?);
        assert_eq!(broker.get_balance("GBP"), Num::from(1));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(1));

        Ok(())
    }

    #[test]
    fn get_updated_limit_buy_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        let order_request = OrderRequest::create_limit_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
            Num::from_str("1.3")?,
        );

        let order_id = broker.place_order(order_request)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id: order_id.clone(),
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.3")?),
                filled_quantity: Num::from(0),
                average_fill_price: None,
                status: OrderStatus::New,
                type_: OrderType::Limit,
                side: OrderSide::Buy,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from_str("14.1")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("1.1")?);
        assert_eq!(broker.get_balance("GBP"), Num::from(0));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(0));

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.29")?)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.3")?),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.29")?),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
                side: OrderSide::Buy,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from_str("1.2")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("1.2")?);
        assert_eq!(broker.get_balance("GBP"), Num::from(10));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(10));

        Ok(())
    }

    #[test]
    fn get_updated_limit_sell_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        broker.update_balance("GBP", Num::from(12));
        broker.update_buying_power("GBP", Num::from(12));

        let order_request = OrderRequest::create_limit_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
            Num::from_str("1.32")?,
        );

        let order_id = broker.place_order(order_request)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id: order_id.clone(),
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.32")?),
                filled_quantity: Num::from(0),
                average_fill_price: None,
                status: OrderStatus::New,
                type_: OrderType::Limit,
                side: OrderSide::Sell,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from(0));
        assert_eq!(broker.get_buying_power("USD"), Num::from(0));
        assert_eq!(broker.get_balance("GBP"), Num::from(12));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(2));

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.33")?)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.32")?),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.33")?),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
                side: OrderSide::Sell,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from_str("13.3")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("13.3")?);
        assert_eq!(broker.get_balance("GBP"), Num::from(2));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(2));

        Ok(())
    }

    #[test]
    fn get_filled_limit_buy_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        let order_request = OrderRequest::create_limit_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
            Num::from_str("1.4")?,
        );

        let order_id = broker.place_order(order_request)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.4")?),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.31")?),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
                side: OrderSide::Buy,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from(1));
        assert_eq!(broker.get_buying_power("USD"), Num::from(1));
        assert_eq!(broker.get_balance("GBP"), Num::from(10));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(10));

        Ok(())
    }

    #[test]
    fn get_filled_limit_sell_order() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD").build();

        broker.set_notional_per_unit(AssetPair::from_str("GBP/USD")?, Num::from_str("1.31")?)?;

        broker.update_balance("GBP", Num::from_str("10.5")?);
        broker.update_buying_power("GBP", Num::from_str("10.5")?);

        let order_request = OrderRequest::create_limit_buy(
            AssetPair::from_str("GBP/USD")?,
            Amount::Quantity {
                quantity: Num::from(10),
            },
            Num::from_str("1.25")?,
        );

        let order_id = broker.place_order(order_request)?;

        let order = broker.get_order(&order_id)?;
        assert_eq!(
            order,
            Order {
                order_id,
                asset_symbol: "GBP/USD".into(),
                amount: Amount::Quantity {
                    quantity: Num::from(10),
                },
                limit_price: Some(Num::from_str("1.25")?),
                filled_quantity: Num::from(10),
                average_fill_price: Some(Num::from_str("1.31")?),
                status: OrderStatus::Filled,
                type_: OrderType::Limit,
                side: OrderSide::Sell,
            }
        );

        assert_eq!(broker.get_balance("USD"), Num::from_str("13.1")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("13.1")?);
        assert_eq!(broker.get_balance("GBP"), Num::from_str("0.5")?);
        assert_eq!(broker.get_buying_power("GBP"), Num::from_str("0.5")?);

        Ok(())
    }

    #[test]
    fn set_notional_per_unit_invalid_notional_asset() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        let err = broker
            .set_notional_per_unit(AssetPair::from_str("GBP/USDT")?, Num::from_str("1.31")?)
            .unwrap_err();

        assert_eq!(err.to_string(), "USDT is not a valid notional asset");

        Ok(())
    }

    #[test]
    fn set_notional_per_unit_inverted_notional_asset() -> Result<()> {
        let mut broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .build();

        let err = broker
            .set_notional_per_unit(AssetPair::from_str("USD/GBP")?, Num::from_str("1.31")?)
            .unwrap_err();

        assert_eq!(err.to_string(), "GBP is not a valid notional asset");

        Ok(())
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
        assert_eq!(broker.get_balance("USD"), Num::from(0));
        assert_eq!(broker.get_buying_power("USD"), Num::from(0));
    }

    #[test]
    fn build_negative_balance() {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from(-10))
            .build();
        assert_eq!(broker.get_balance("USD"), Num::from(-10));
        assert_eq!(broker.get_buying_power("USD"), Num::from(-10));
    }

    #[test]
    fn build_with_notional_assets() -> Result<()> {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from_str("14.1")?)
            .add_notional_asset("BTC", None)
            .add_notional_asset("USDT", Some(Num::from(-10)))
            .build();

        assert_eq!(
            broker.get_balance(&broker.get_currency()),
            Num::from_str("14.1")?
        );
        assert_eq!(
            broker.get_buying_power(&broker.get_currency()),
            Num::from_str("14.1")?
        );
        assert_eq!(broker.get_balance("USD"), Num::from_str("14.1")?);
        assert_eq!(broker.get_buying_power("USD"), Num::from_str("14.1")?);
        assert_eq!(broker.get_balance("USDT"), Num::from(-10));
        assert_eq!(broker.get_buying_power("USDT"), Num::from(-10));
        assert_eq!(broker.get_balance("BTC"), Num::from(0));
        assert_eq!(broker.get_buying_power("BTC"), Num::from(0));
        assert_eq!(broker.get_balance("GBP"), Num::from(0));
        assert_eq!(broker.get_buying_power("GBP"), Num::from(0));

        Ok(())
    }
}
