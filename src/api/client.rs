// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Account, Order};
use crate::api::request::OrderRequest;
use anyhow::Result;

/// A trait for instances of a trading client, which allows operations with the underlying trading broker.
pub trait Client {
    /// Places an order. If successful returns the order id of the newly placed order.
    fn place_order(&mut self, req: OrderRequest) -> impl Future<Output = Result<String>> + Send;

    /// Returns all placed orders.
    fn get_orders(&self) -> impl Future<Output = Result<Vec<Order>>> + Send;

    /// Returns an order by id.
    fn get_order(&self, order_id: &str) -> impl Future<Output = Result<Order>> + Send;

    /// Returns the account associated with this client.
    fn get_account(&self) -> impl Future<Output = Result<Account>> + Send;
}
