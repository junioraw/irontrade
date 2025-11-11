// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::request::{BuyMarketRequest, SellMarketRequest};
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse,
};
use crate::provider::IronTradeClientProvider;
use crate::providers::simulated::broker::SimulatedBroker;
use anyhow::Result;
use num_decimal::Num;
use std::collections::HashMap;

mod broker;
mod simulated;

pub struct SimulatedIronTradeClientProvider {
    usd_balance: Num,
}

impl SimulatedIronTradeClientProvider {
    pub fn new(usd_balance: Num) -> Self {
        Self { usd_balance }
    }
}

impl IronTradeClientProvider<SimulatedIronTradeClient> for SimulatedIronTradeClientProvider {
    fn create_client(&self) -> Result<SimulatedIronTradeClient> {
        let mut balance = HashMap::new();
        balance.insert("USD".into(), self.usd_balance.clone());
        let broker = SimulatedBroker::new(balance);
        Ok(SimulatedIronTradeClient { broker })
    }
}

pub struct SimulatedIronTradeClient {
    broker: SimulatedBroker,
}

impl IronTradeClient for SimulatedIronTradeClient {
    async fn buy_market(&self, req: BuyMarketRequest) -> Result<BuyMarketResponse> {
        todo!()
    }

    async fn sell_market(&self, req: SellMarketRequest) -> Result<SellMarketResponse> {
        todo!()
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        todo!()
    }
}
