// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::request::{Amount, BuyMarketRequest, SellMarketRequest};
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOpenPositionsResponse, GetOrderResponse,
    GetOrdersResponse,
};
use crate::provider::IronTradeClientProvider;
use anyhow::Result;
use apca::api::v2::order;
use apca::api::v2::order::Amount as ApcaAmount;
use apca::api::v2::order::{Side, Type};
use apca::{ApiInfo, Client};

pub struct AlpacaIronTradeClientProvider {
    api_info: ApiInfo,
}

impl AlpacaIronTradeClientProvider {
    pub fn new(api_info: ApiInfo) -> Self {
        Self { api_info }
    }
}

impl IronTradeClientProvider<AlpacaIronTradeClient> for AlpacaIronTradeClientProvider {
    async fn create_client(&self) -> Result<AlpacaIronTradeClient> {
        Ok(AlpacaIronTradeClient::new(self.api_info.clone()))
    }
}

struct AlpacaIronTradeClient {
    apca_client: Client,
}

impl AlpacaIronTradeClient {
    fn new(api_info: ApiInfo) -> Self {
        Self {
            apca_client: Client::new(api_info),
        }
    }
}

impl IronTradeClient for AlpacaIronTradeClient {
    async fn buy_market(&self, req: BuyMarketRequest) -> Result<BuyMarketResponse> {
        let request = order::CreateReqInit {
            type_: Type::Market,
            ..Default::default()
        }
        .init(req.asset_symbol, Side::Buy, to_apca_amount(req.amount));

        let order = self.apca_client.issue::<order::Create>(&request).await?;

        Ok(BuyMarketResponse {
            order_id: order.id.to_string(),
        })
    }

    async fn sell_market(&self, req: SellMarketRequest) -> Result<SellMarketRequest> {
        todo!()
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        todo!()
    }

    async fn get_order(&self, order_id: String) -> Result<GetOrderResponse> {
        todo!()
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        todo!()
    }

    async fn get_open_positions(&self) -> Result<GetOpenPositionsResponse> {
        todo!()
    }
}

fn to_apca_amount(amount: Amount) -> ApcaAmount {
    match amount {
        Amount::Quantity { quantity } => ApcaAmount::Quantity { quantity },
        Amount::Notional { notional } => ApcaAmount::Notional { notional },
    }
}
