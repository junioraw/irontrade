use crate::api::client::IronTradeClient;
use crate::api::common::{Amount, AssetPair};
use crate::api::request::MarketOrderRequest;
use crate::api::response::{
    GetCashResponse, GetOpenPositionResponse, GetOrdersResponse, MarketOrderResponse, OpenPosition,
};
use crate::util::simulated::broker::SimulatedBroker;
use anyhow::Result;

pub struct SimulatedClient {
    broker: SimulatedBroker,
}

impl IronTradeClient for SimulatedClient {
    async fn buy_market(&mut self, req: MarketOrderRequest) -> Result<MarketOrderResponse> {
        let order_id = self.broker.place_order(req)?;
        Ok(MarketOrderResponse { order_id })
    }

    async fn sell_market(&mut self, req: MarketOrderRequest) -> Result<MarketOrderResponse> {
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
        Ok(MarketOrderResponse { order_id })
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
        Ok(GetCashResponse {
            cash: self.broker.get_balance(&self.broker.get_currency()),
        })
    }

    async fn get_open_position(&self, asset_symbol: &str) -> Result<GetOpenPositionResponse> {
        let balance = self.broker.get_balance(asset_symbol);
        let notional_per_unit = self.broker.get_notional_per_unit(&AssetPair {
            notional_asset: self.broker.get_currency(),
            quantity_asset: asset_symbol.into(),
        })?;
        Ok(GetOpenPositionResponse {
            open_position: OpenPosition {
                asset_symbol: asset_symbol.into(),
                quantity: balance.clone(),
                average_entry_price: None,
                market_value: Some(balance * notional_per_unit),
            },
        })
    }
}
