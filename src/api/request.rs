// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use crate::api::common::{Amount, AssetPair, OrderSide};

pub struct OrderRequestV1 {
    pub asset_pair: AssetPair,
    pub amount: Amount,
    pub limit_price: Option<Num>
}

pub struct OrderRequest {
    pub asset_pair: AssetPair,
    pub amount: Amount,
    pub limit_price: Option<Num>,
    pub side: OrderSide,
}

