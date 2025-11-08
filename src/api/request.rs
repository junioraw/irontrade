// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;

pub struct BuyMarketRequest {
    pub asset_symbol: String,
    pub amount: Amount,
}

pub struct SellMarketRequest {
    asset_symbol: String,
    amount: Amount,
}

pub enum Amount {
    Quantity { quantity: Num },
    Notional { notional: Num },
}