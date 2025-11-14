// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::string::ParseError;
use num_decimal::Num;

pub enum Amount {
    Quantity { quantity: Num },
    Notional { notional: Num },
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub struct AssetPair {
    pub notional_asset: String,
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