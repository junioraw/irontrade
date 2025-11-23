// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;
use crate::api::common::{OpenPosition, Order, OrderV1};

pub struct OrderResponse {
    pub order_id: String,
}

pub struct GetOrdersResponseV1 {
    pub orders: Vec<OrderV1>,
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
