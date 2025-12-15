// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::common::{Account, CryptoPair, Bar, Order};
use crate::api::environment::Environment;
use crate::api::market::Market;
use crate::api::request::OrderRequest;
use crate::simulated::client::SimulatedClient;
use crate::simulated::data::BarDataSource;
use crate::simulated::time::Clock;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;

pub struct SimulatedEnvironment {
    client: SimulatedClient,
    bar_data_source: Box<dyn BarDataSource + Send + Sync>,
    last_processed_time: Option<DateTime<Utc>>,
    assets_to_trade: HashSet<CryptoPair>,
    clock: Box<dyn Clock + Send + Sync>,
}

impl SimulatedEnvironment {
    pub fn init(&mut self) -> Result<()> {
        if self.last_processed_time.is_some() {
            return Err(anyhow!("Environment has already been initialized"));
        }
        self.update()
    }

    fn update(&mut self) -> Result<()> {
        if self.last_processed_time.is_none() {
            return Err(anyhow!("Environment has not been initialized"));
        }
        let now = self.clock.now();
        let period = Duration::seconds(30);
        let mut last_processed_time = self.last_processed_time.unwrap_or(now);
        while last_processed_time <= now {
            let assets_to_trade = self.assets_to_trade.clone();
            for asset_pair in assets_to_trade {
                let bar = self.bar_data_source.get_bar(&asset_pair, &now)?;
                if let Some(bar) = bar {
                    let value = (bar.low + bar.high) / 2.0;
                    self.client.set_notional_per_unit(asset_pair, value)?;
                }
            }
            if last_processed_time == now {
                break;
            }
            last_processed_time = DateTime::min(last_processed_time + period, now);
        }
        self.last_processed_time = Some(now);
        Ok(())
    }
}

impl Client for SimulatedEnvironment {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String> {
        self.update()?;
        self.client.place_order(req).await
    }

    async fn get_orders(&mut self) -> Result<Vec<Order>> {
        self.update()?;
        self.client.get_orders().await
    }

    async fn get_order(&mut self, order_id: &str) -> Result<Order> {
        self.update()?;
        self.client.get_order(order_id).await
    }

    async fn get_account(&mut self) -> Result<Account> {
        self.update()?;
        self.client.get_account().await
    }
}

impl Market for SimulatedEnvironment {
    async fn get_latest_bar(&self, asset_pair: &CryptoPair) -> Result<Option<Bar>> {
        self.bar_data_source.get_bar(asset_pair, &self.clock.now())
    }
}

impl Environment for SimulatedEnvironment {}
