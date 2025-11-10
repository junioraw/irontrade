// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::request::{BuyMarketRequest, SellMarketRequest};
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse,
};
use anyhow::Result;

pub trait IronTradeClient {
    fn buy_market(
        &self,
        req: BuyMarketRequest,
    ) -> impl Future<Output = Result<BuyMarketResponse>> + Send;
    fn sell_market(
        &self,
        req: SellMarketRequest,
    ) -> impl Future<Output = Result<SellMarketResponse>> + Send;
    fn get_orders(&self) -> impl Future<Output = Result<GetOrdersResponse>> + Send;
    fn get_open_position(
        &self,
        asset_symbol: String,
    ) -> impl Future<Output = Result<GetOpenPositionResponse>> + Send;
}
