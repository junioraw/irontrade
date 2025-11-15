// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::Amount;
use num_decimal::Num;

pub struct BuyMarketResponse {
    pub order_id: String,
}

pub struct SellMarketResponse {
    pub order_id: String,
}

pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
}

pub struct GetCashResponse {
    pub cash: Num,
}

pub struct GetOpenPositionResponse {
    pub open_position: OpenPosition,
}

pub struct Order {
    pub order_id: String,
    pub asset_symbol: String,
    pub amount: Amount,
    pub filled_quantity: Num,
    pub average_fill_price: Option<Num>,
    pub status: OrderStatus,
    pub type_: OrderType,
}

pub struct OpenPosition {
    pub asset_symbol: String,
    pub average_entry_price: Num,
    pub quantity: Num,
    pub market_value: Option<Num>,
}

pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Expired,
    Unimplemented,
}

pub enum OrderType {
    Market,
    Limit,
}
