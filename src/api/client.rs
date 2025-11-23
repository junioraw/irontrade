// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::request::{OrderRequest, OrderRequestV1};
use crate::api::response::{
    GetCashResponse, GetOpenPositionResponse, GetOrdersResponse, GetOrdersResponseV1, OrderResponse,
};
use anyhow::Result;

/// A trait for instances of a trading client, which allows operations with the underlying trading broker.
pub trait IronTradeClient {
    /// Places an order. If successful returns the order id of the newly placed order.
    fn place_order(&mut self, req: OrderRequest) -> impl Future<Output = Result<OrderResponse>>;

    /// Places a buy order. If successful returns the order id of the newly placed order.
    fn buy(&mut self, req: OrderRequestV1) -> impl Future<Output = Result<OrderResponse>> + Send;

    /// Places a sell order. If successful returns the order of the newly placed order.
    fn sell(&mut self, req: OrderRequestV1) -> impl Future<Output = Result<OrderResponse>> + Send;

    /// Returns all placed orders.
    fn get_orders_v1(&self) -> impl Future<Output = Result<GetOrdersResponseV1>> + Send;

    /// Returns all placed orders.
    fn get_orders(&self) -> impl Future<Output = Result<GetOrdersResponse>> + Send;

    /// Returns the current cash balance, more specifically the balance of the currency
    /// that is not tied to any order or open position.
    fn get_cash(&self) -> impl Future<Output = Result<GetCashResponse>> + Send;

    /// Returns an open position for the given asset_symbol.
    fn get_open_position(
        &self,
        asset_symbol: &str,
    ) -> impl Future<Output = Result<GetOpenPositionResponse>> + Send;
}
