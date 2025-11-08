// SPDX-License-Identifier: GPL-3.0-or-later

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