// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::request::MarketOrderRequest;
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse,
};
use anyhow::Result;

/// A trait for instances of a trading client, which allows operations with the underlying trading broker.
pub trait IronTradeClient {

    /// Places a market buy order. If successful returns the order id of the newly placed order.
    fn buy_market(
        &mut self,
        req: MarketOrderRequest,
    ) -> impl Future<Output = Result<BuyMarketResponse>> + Send;

    /// Places a market sell order. If successful returns the order of the newly placed order.
    fn sell_market(
        &mut self,
        req: MarketOrderRequest,
    ) -> impl Future<Output = Result<SellMarketResponse>> + Send;

    /// Returns all placed orders regardless of status.
    fn get_orders(&self) -> impl Future<Output = Result<GetOrdersResponse>> + Send;

    /// Returns an open position for the given asset_symbol.
    fn get_open_position(
        &self,
        asset_symbol: String,
    ) -> impl Future<Output = Result<GetOpenPositionResponse>> + Send;
}
