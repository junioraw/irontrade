// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ParseError;

/// Enum representing different value types, can either be a quantity or a notional value.
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
/// #Examples
///
/// ```
/// use irontrade::api::common::AssetPair;
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
