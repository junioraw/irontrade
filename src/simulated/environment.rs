// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::common::{Account, AssetPair, Bar, Order};
use crate::api::environment::Environment;
use crate::api::market::Market;
use crate::api::request::OrderRequest;
use crate::simulated::client::SimulatedClient;
use crate::simulated::data::BarDataSource;
use crate::simulated::time::Clock;
use anyhow::Result;

pub struct SimulatedEnvironment {
    client: SimulatedClient,
    bar_data_source: Box<dyn BarDataSource + Send + Sync>,
    clock: Box<dyn Clock + Send + Sync>,
}

impl Client for SimulatedEnvironment {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String> {
        self.client.place_order(req).await
    }

    async fn get_orders(&self) -> Result<Vec<Order>> {
        self.client.get_orders().await
    }

    async fn get_order(&self, order_id: &str) -> Result<Order> {
        self.client.get_order(order_id).await
    }

    async fn get_account(&self) -> Result<Account> {
        self.client.get_account().await
    }
}

impl Market for SimulatedEnvironment {
    async fn get_latest_bar(&self, asset_pair: &AssetPair) -> Result<Option<Bar>> {
        self.bar_data_source.get_bar(asset_pair, &self.clock.now())
    }
}

impl Environment for SimulatedEnvironment {}
