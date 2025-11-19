// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use crate::api::common::{OpenPosition, Order};

pub struct MarketOrderResponse {
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
