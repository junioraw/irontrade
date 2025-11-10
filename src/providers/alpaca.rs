// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use crate::api::request::{Amount, BuyMarketRequest, SellMarketRequest};
use crate::api::response::{
    BuyMarketResponse, GetOpenPositionResponse, GetOrdersResponse,
    OpenPosition, Order, OrderStatus, OrderType, SellMarketResponse,
};
use crate::provider::IronTradeClientProvider;
use anyhow::Result;
use apca::api::v2::asset::Symbol;
use apca::api::v2::order::Amount as ApcaAmount;
use apca::api::v2::order::Order as ApcaOrder;
use apca::api::v2::order::Status as ApcaOrderStatus;
use apca::api::v2::order::{Side, Type};
use apca::api::v2::orders::ListReq;
use apca::api::v2::position::Position;
use apca::api::v2::{order, orders, position};
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

    async fn sell_market(&self, req: SellMarketRequest) -> Result<SellMarketResponse> {
        let request = order::CreateReqInit {
            type_: Type::Market,
            ..Default::default()
        }
        .init(req.asset_symbol, Side::Sell, to_apca_amount(req.amount));

        let order = self.apca_client.issue::<order::Create>(&request).await?;

        Ok(SellMarketResponse {
            order_id: order.id.to_string(),
        })
    }

    async fn get_orders(&self) -> Result<GetOrdersResponse> {
        let orders: Vec<Order> = self
            .apca_client
            .issue::<orders::List>(&ListReq {
                ..Default::default()
            })
            .await?
            .iter()
            .map(|order| from_apca_order(order.clone()))
            .collect();

        Ok(GetOrdersResponse { orders })
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        let position = self
            .apca_client
            .issue::<position::Get>(&Symbol::Sym(asset_symbol))
            .await?;

        Ok(GetOpenPositionResponse {
            open_position: from_apca_position(position),
        })
    }
}

fn to_apca_amount(amount: Amount) -> ApcaAmount {
    match amount {
        Amount::Quantity { quantity } => ApcaAmount::Quantity { quantity },
        Amount::Notional { notional } => ApcaAmount::Notional { notional },
    }
}

fn from_apca_amount(amount: ApcaAmount) -> Amount {
    match amount {
        ApcaAmount::Quantity { quantity } => Amount::Quantity { quantity },
        ApcaAmount::Notional { notional } => Amount::Notional { notional },
    }
}

fn from_apca_position(position: Position) -> OpenPosition {
    OpenPosition {
        asset_symbol: position.symbol.to_string(),
        average_entry_price: position.average_entry_price,
        quantity: position.quantity,
        market_value: position.market_value,
    }
}

fn from_apca_order_status(status: ApcaOrderStatus) -> OrderStatus {
    match status {
        ApcaOrderStatus::New => OrderStatus::New,
        ApcaOrderStatus::PartiallyFilled => OrderStatus::PartiallyFilled,
        ApcaOrderStatus::Filled => OrderStatus::Filled,
        ApcaOrderStatus::Expired => OrderStatus::Expired,
        _ => todo!(),
    }
}

fn from_apca_order_type(type_: Type) -> OrderType {
    match type_ {
        Type::Market => OrderType::Market,
        Type::Limit => OrderType::Limit,
        _ => todo!(),
    }
}

fn from_apca_order(order: ApcaOrder) -> Order {
    Order {
        order_id: order.id.to_string(),
        asset_symbol: order.symbol,
        filled_quantity: order.filled_quantity,
        amount: from_apca_amount(order.amount),
        average_fill_price: order.average_fill_price,
        status: from_apca_order_status(order.status),
        type_: from_apca_order_type(order.type_),
    }
}
