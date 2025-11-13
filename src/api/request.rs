// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::Amount;

pub struct BuyMarketRequest {
    pub asset_symbol: String,
    pub amount: Amount,
}

pub struct SellMarketRequest {
    pub asset_symbol: String,
    pub amount: Amount,
}

