// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;

pub struct BuyMarketRequest {
    asset_symbol: String,
    amount: Amount,
}

pub struct SellMarketRequest {
    asset_symbol: String,
    amount: Amount,
}

enum Amount {
    Quantity { quantity: Num },
    Notional { notional: Num },
}