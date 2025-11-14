// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::common::Amount;
use crate::api::request::MarketOrderRequest;
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse,
};
use crate::provider::IronTradeClientProvider;
use crate::providers::simulated::broker::{OrderRequest, SimulatedBroker};
use anyhow::Result;
use num_decimal::Num;
use std::collections::HashMap;

mod broker;

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
        let mut balances = HashMap::new();
        balances.insert("USD".into(), self.usd_balance.clone());
        let broker = SimulatedBroker::new(balances);
        Ok(SimulatedIronTradeClient { broker })
    }
}

pub struct SimulatedIronTradeClient {
    broker: SimulatedBroker,
}

impl IronTradeClient for SimulatedIronTradeClient {
    async fn buy_market(&mut self, req: MarketOrderRequest) -> Result<BuyMarketResponse> {
        let quantity_to_buy: Num;
        let max_price: Num = self.broker.get_exchange_rate(&req.asset_pair)?;

        match req.amount {
            Amount::Quantity { quantity } => {
                quantity_to_buy = quantity;
            }
            Amount::Notional { notional } => {
                quantity_to_buy = notional * &max_price;
            }
        }

        let order_id = self.broker.place_order(OrderRequest {
            asset_pair: req.asset_pair,
            amount: Amount::Quantity {
                quantity: quantity_to_buy,
            },
        })?;

        Ok(BuyMarketResponse { order_id })
    }

    async fn sell_market(&mut self, req: MarketOrderRequest) -> Result<SellMarketResponse> {
        todo!()
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        todo!()
    }
}
