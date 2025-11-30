// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::request::OrderRequest;
use anyhow::Result;
use num_decimal::Num;
use crate::api::common::{OpenPosition, Order};

/// A trait for instances of a trading client, which allows operations with the underlying trading broker.
pub trait Client {
    /// Places an order. If successful returns the order id of the newly placed order.
    fn place_order(&mut self, req: OrderRequest) -> impl Future<Output = Result<String>> + Send;

    /// Returns all placed orders.
    fn get_orders(&self) -> impl Future<Output = Result<Vec<Order>>> + Send;

    /// Returns buying power.
    fn get_buying_power(&self) -> impl Future<Output = Result<Num>> + Send;

    /// Returns the current cash balance, more specifically the balance of the currency
    /// that is not tied to any order or open position.
    fn get_cash(&self) -> impl Future<Output = Result<Num>> + Send;

    /// Returns an open position for the given asset_symbol.
    fn get_open_position(
        &self,
        asset_symbol: &str,
    ) -> impl Future<Output = Result<OpenPosition>> + Send;
}
