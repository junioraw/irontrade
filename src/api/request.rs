// SPDX-License-Identifier: GPL-3.0-or-later

use bigdecimal::BigDecimal;
use crate::api::common::{Amount, CryptoPair, OrderSide};

pub struct OrderRequest {
    pub asset_pair: CryptoPair,
    pub amount: Amount,
    pub limit_price: Option<BigDecimal>,
    pub side: OrderSide,
}

impl OrderRequest {
    pub fn create_market_buy(asset_pair: CryptoPair, amount: Amount) -> Self {
        OrderRequest {
            asset_pair,
            amount,
            limit_price: None,
            side: OrderSide::Buy,
        }
    }

    pub fn create_market_sell(asset_pair: CryptoPair, amount: Amount) -> Self {
        OrderRequest {
            asset_pair,
            amount,
            limit_price: None,
            side: OrderSide::Sell,
        }
    }

    pub fn create_limit_buy(asset_pair: CryptoPair, amount: Amount, limit_price: BigDecimal) -> Self {
        OrderRequest {
            asset_pair,
            amount,
            limit_price: Some(limit_price),
            side: OrderSide::Buy,
        }
    }

    pub fn create_limit_sell(asset_pair: CryptoPair, amount: Amount, limit_price: BigDecimal) -> Self {
        OrderRequest {
            asset_pair,
            amount,
            limit_price: Some(limit_price),
            side: OrderSide::Sell,
        }
    }
}
