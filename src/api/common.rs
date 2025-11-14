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
    pub asset_on_sale: String,
    pub asset_being_bought: String,
}

impl AssetPair {
    pub fn inverse(&self) -> AssetPair {
        AssetPair {
            asset_on_sale: self.asset_being_bought.clone(),
            asset_being_bought: self.asset_on_sale.clone(),
        }
    }
}

impl FromStr for AssetPair {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.split("/").collect();
        Ok(AssetPair {
            asset_on_sale: tokens[1].into(),
            asset_being_bought: tokens[0].into(),
        })
    }
}
impl Display for AssetPair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}/{}",
            self.asset_being_bought, self.asset_on_sale
        ))
    }
}