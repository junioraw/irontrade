// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Account, Order};
use crate::api::request::OrderRequest;
use anyhow::Result;

pub trait Client {
    fn place_order(&mut self, req: OrderRequest) -> impl Future<Output = Result<String>> + Send;

    fn get_orders(&mut self) -> impl Future<Output = Result<Vec<Order>>> + Send;

    fn get_order(&mut self, order_id: &str) -> impl Future<Output = Result<Order>> + Send;

    fn get_account(&mut self) -> impl Future<Output = Result<Account>> + Send;
}
