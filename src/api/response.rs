// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;

pub struct BuyMarketResponse {
    pub order_id: String,
}

pub struct SellMarketResponse {
    pub order_id: String,
}

pub struct GetOrderResponse {
    pub order: Order,
}

pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
}

pub struct GetOpenPositionResponse {
    pub position: OpenPosition,
}

pub struct GetOpenPositionsResponse {
    pub positions: Vec<OpenPosition>,
}

pub struct Order {
    pub order_id: String,
    pub asset_symbol: String,
    pub quantity: Num,
    pub notional: Num,
    pub filled_quantity: Num,
    pub filled_avg_price: Num,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
}

pub struct OpenPosition {
    pub asset_symbol: String,
    pub avg_entry_price: Num,
    pub quantity: Num,
    pub market_value: Num,
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