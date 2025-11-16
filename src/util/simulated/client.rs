use crate::api::client::IronTradeClient;
use crate::api::common::Amount;
use crate::api::request::MarketOrderRequest;
use crate::api::response::{BuyMarketResponse, GetCashResponse, GetOpenPositionResponse, GetOrdersResponse, SellMarketResponse};
use crate::util::simulated::broker::SimulatedBroker;
use anyhow::Result;

pub struct SimulatedClient {
    broker: SimulatedBroker,
}

impl IronTradeClient for SimulatedClient {
    async fn buy_market(&mut self, req: MarketOrderRequest) -> Result<BuyMarketResponse> {
        let order_id = self.broker.place_order(req)?;
        Ok(BuyMarketResponse { order_id })
    }

    async fn sell_market(&mut self, req: MarketOrderRequest) -> Result<SellMarketResponse> {
        let req = MarketOrderRequest {
            asset_pair: req.asset_pair,
            amount: match req.amount {
                Amount::Quantity { quantity } => Amount::Quantity {
                    quantity: -quantity,
                },
                Amount::Notional { notional } => Amount::Notional {
                    notional: -notional,
                },
            },
        };
        let order_id = self.broker.place_order(req)?;
        Ok(SellMarketResponse { order_id })
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        Ok(GetOrdersResponse {
            orders: self
                .broker
                .get_orders()
                .iter()
                .map(|order| order.clone().into())
                .collect(),
        })
    }

    async fn get_cash(&self) -> Result<GetCashResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: &str) -> Result<GetOpenPositionResponse> {
        todo!()
    }
}