// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair};

pub struct BuyMarketRequest {
    pub asset_pair: AssetPair,
    pub amount: Amount,
}

pub struct SellMarketRequest {
    pub asset_pair: AssetPair,
    pub amount: Amount,
}

