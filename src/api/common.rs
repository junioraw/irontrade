// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ParseError;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};

pub struct Account {
    pub open_positions: HashMap<String, OpenPosition>,
    pub cash: BigDecimal,
    pub currency: String,
    pub buying_power: BigDecimal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub order_id: String,
    pub asset_symbol: String,
    pub amount: Amount,
    pub limit_price: Option<BigDecimal>,
    pub filled_quantity: BigDecimal,
    pub average_fill_price: Option<BigDecimal>,
    pub status: OrderStatus,
    pub type_: OrderType,
    pub side: OrderSide,
}

#[derive(PartialEq, Eq, Debug)]
pub struct OpenPosition {
    pub asset_symbol: String,
    pub average_entry_price: Option<BigDecimal>,
    pub quantity: BigDecimal,
    pub market_value: Option<BigDecimal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Expired,
    Unimplemented,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum Amount {
    Quantity { quantity: BigDecimal },
    Notional { notional: BigDecimal },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct AssetPair {
    pub notional_asset: String,
    pub quantity_asset: String,
}

pub struct Bar {
    pub low: BigDecimal,
    pub high: BigDecimal,
    pub open: BigDecimal,
    pub close: BigDecimal,
    pub date_time: DateTime<Utc>,
}

impl FromStr for AssetPair {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.split("/").collect();
        Ok(AssetPair {
            notional_asset: tokens[1].into(),
            quantity_asset: tokens[0].into(),
        })
    }
}
impl Display for AssetPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}/{}",
            self.quantity_asset, self.notional_asset
        ))
    }
}
