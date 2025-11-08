// SPDX-License-Identifier: GPL-3.0-or-later

pub mod client {
    use crate::api::request::{BuyMarketRequest, SellMarketRequest};
    use crate::api::response::{
        BuyMarketResponse, GetOpenPositionResponse, GetOpenPositionsResponse, GetOrderResponse,
        GetOrdersResponse,
    };

    trait IronTradeClient {
        async fn buy_market(req: BuyMarketRequest) -> anyhow::Result<BuyMarketResponse>;
        async fn sell_market(req: SellMarketRequest) -> anyhow::Result<SellMarketRequest>;
        async fn get_orders() -> anyhow::Result<GetOrdersResponse>;
        async fn get_order(order_id: String) -> anyhow::Result<GetOrderResponse>;
        async fn get_open_position(asset_symbol: String)
                                   -> anyhow::Result<GetOpenPositionResponse>;
        async fn get_open_positions() -> anyhow::Result<GetOpenPositionsResponse>;
    }
}

pub mod request {
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
}

pub mod response {
    use num_decimal::Num;

    pub struct BuyMarketResponse {
        order_id: String,
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
}