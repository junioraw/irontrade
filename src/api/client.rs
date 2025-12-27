// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Account, Order};
use crate::api::request::OrderRequest;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Client {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String>;

    async fn get_orders(&mut self) -> Result<Vec<Order>>;

    async fn get_order(&mut self, order_id: &str) -> Result<Order>;

    async fn get_account(&mut self) -> Result<Account>;
}
