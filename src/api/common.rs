// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ParseError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub order_id: String,
    pub asset_symbol: String,
    pub amount: Amount,
    pub filled_quantity: Num,
    pub average_fill_price: Option<Num>,
    pub status: OrderStatus,
    pub type_: OrderType,
    pub side: OrderSide,
}

#[derive(PartialEq, Eq, Debug)]
pub struct OpenPosition {
    pub asset_symbol: String,
    pub average_entry_price: Option<Num>,
    pub quantity: Num,
    pub market_value: Option<Num>,
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

/// Enum representing different value types, can either be a quantity or a notional value.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum Amount {
    /// Quantity, usually amount of non-notional assets.
    Quantity { quantity: Num },
    /// Notional, usually amount of a notional asset, for example a currency.
    Notional { notional: Num },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
/// Struct defining an asset pair, usually used in buy/sell orders.
///
/// Can either be initialized via struct construction or from parsing a string representing an asset pair.
///
/// #Example
///
/// ```
/// use irontrade::api::common::AssetPair;
/// use std::str::FromStr;
///
/// let asset_pair = AssetPair::from_str("BTC/USD").unwrap();
/// ```
pub struct AssetPair {
    /// A notional asset, like a currency or a tethered crypto coin.
    pub notional_asset: String,
    /// A quantifiable asset, like a stock or a crypto coin.
    pub quantity_asset: String,
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
