// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::request::{BuyMarketRequest, SellMarketRequest};
use crate::api::response::{BuyMarketResponse, GetOpenPositionResponse, GetOpenPositionsResponse, GetOrderResponse, GetOrdersResponse};

struct AlpacaTradeClient {}

impl IronTradeClient for AlpacaTradeClient {
    async fn buy_market(&self, req: BuyMarketRequest) -> anyhow::Result<BuyMarketResponse> {
        todo!()
    }

    async fn sell_market(&self, req: SellMarketRequest) -> anyhow::Result<SellMarketRequest> {
        todo!()
    }

    async fn get_orders(&self) -> anyhow::Result<GetOrdersResponse> {
        todo!()
    }

    async fn get_order(&self, order_id: String) -> anyhow::Result<GetOrderResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: String) -> anyhow::Result<GetOpenPositionResponse> {
        todo!()
    }

    async fn get_open_positions(&self) -> anyhow::Result<GetOpenPositionsResponse> {
        todo!()
    }
}