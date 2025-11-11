// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::request::{Amount, BuyMarketRequest, SellMarketRequest};
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse,
};
use crate::provider::IronTradeClientProvider;
use crate::providers::simulated::broker::{AssetPair, OrderRequest, SimulatedBroker};
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
    async fn buy_market(&mut self, req: BuyMarketRequest) -> Result<BuyMarketResponse> {
        let from_asset: String;
        let to_asset: String;

        if req.asset_symbol.contains("/") {
            let mut assets = req.asset_symbol.split("/");
            to_asset = assets.next().unwrap().to_string();
            from_asset = assets.next().unwrap().to_string();
        } else {
            from_asset = "USD".into();
            to_asset = req.asset_symbol.to_string();
        }

        let asset_pair = AssetPair { from_asset, to_asset };

        let quantity_to_buy: Num;
        let max_price: Num = self.broker.get_exchange_rate(&asset_pair).unwrap();

        match req.amount {
            Amount::Quantity { quantity } => {
                quantity_to_buy = quantity;
            }
            Amount::Notional { notional } => {
                quantity_to_buy = notional * &max_price;
            }
        }

        let order_id = self.broker.place_order( OrderRequest {
            asset_pair,
            quantity_to_buy,
            max_price,
        })?;

        Ok(BuyMarketResponse {
            order_id,
        })
    }

    async fn sell_market(&mut self, req: SellMarketRequest) -> Result<SellMarketResponse> {
        todo!()
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        todo!()
    }
}
