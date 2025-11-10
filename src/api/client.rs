// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::request::{BuyMarketRequest, SellMarketRequest};
use crate::api::response::{BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse};
use anyhow::Result;

pub trait IronTradeClient {
    async fn buy_market(&self, req: BuyMarketRequest) -> Result<BuyMarketResponse>;
    async fn sell_market(&self, req: SellMarketRequest) -> Result<SellMarketRequest>;
    async fn get_orders(&self) -> Result<GetOrdersResponse>;
    async fn get_open_position(
        &self,
        asset_symbol: String,
    ) -> Result<GetOpenPositionResponse>;
}
