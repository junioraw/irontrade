// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;

pub struct BuyMarketResponse {
    pub order_id: String,
}

pub struct SellMarketResponse {
    order_id: String,
}

pub struct GetOrderResponse {
    order: Order,
}

pub struct GetOrdersResponse {
    orders: Vec<Order>,
}

pub struct GetOpenPositionResponse {
    position: OpenPosition,
}

pub struct GetOpenPositionsResponse {
    position: Vec<OpenPosition>,
}

pub struct Order {
    order_id: String,
    asset_symbol: String,
    quantity: Num,
    notional: Num,
    filled_quantity: Num,
    filled_avg_price: Num,
    order_status: OrderStatus,
    order_type: OrderType,
}

pub struct OpenPosition {
    asset_symbol: String,
    avg_entry_price: Num,
    quantity: Num,
    market_value: Num,
}

pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Expired,
}

pub enum OrderType {
    Buy,
    Sell,
}