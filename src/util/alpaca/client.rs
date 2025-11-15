use crate::api::client::IronTradeClient;
use crate::api::provider::IronTradeClientBuilder;
use crate::api::request::MarketOrderRequest;
use crate::api::response::{
    BuyMarketResponse, GetCashResponse, GetOpenPositionResponse, GetOrdersResponse, Order,
    SellMarketResponse,
};
use anyhow::Result;
use apca::api::v2::asset::Symbol;
use apca::api::v2::order::{Side, TimeInForce, Type};
use apca::api::v2::orders::ListReq;
use apca::api::v2::{account, order, orders, position};
use apca::{ApiInfo, Client};

pub struct AlpacaClient {
    apca_client: Client,
}

impl AlpacaClient {
    fn new(api_info: ApiInfo) -> Self {
        Self {
            apca_client: Client::new(api_info),
        }
    }
}

pub struct AlpacaClientBuilder {
    api_info: ApiInfo,
}

impl AlpacaClientBuilder {
    pub fn new(api_info: ApiInfo) -> Self {
        Self { api_info }
    }
}

impl IronTradeClientBuilder<AlpacaClient> for AlpacaClientBuilder {
    fn build(self) -> Result<AlpacaClient> {
        Ok(AlpacaClient::new(self.api_info))
    }
}

impl IronTradeClient for AlpacaClient {
    async fn buy_market(&mut self, req: MarketOrderRequest) -> Result<BuyMarketResponse> {
        let request = order::CreateReqInit {
            type_: Type::Market,
            time_in_force: TimeInForce::UntilCanceled,
            ..Default::default()
        }
        .init(req.asset_pair.to_string(), Side::Buy, req.amount.into());

        let order = self.apca_client.issue::<order::Create>(&request).await?;

        Ok(BuyMarketResponse {
            order_id: order.id.to_string(),
        })
    }

    async fn sell_market(&mut self, req: MarketOrderRequest) -> Result<SellMarketResponse> {
        let request = order::CreateReqInit {
            type_: Type::Market,
            ..Default::default()
        }
        .init(req.asset_pair.to_string(), Side::Sell, req.amount.into());

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
            .map(|order| order.clone().into())
            .collect();

        Ok(GetOrdersResponse { orders })
    }

    async fn get_cash(&self) -> Result<GetCashResponse> {
        let account = self.apca_client.issue::<account::Get>(&()).await?;
        Ok(GetCashResponse { cash: account.cash })
    }

    async fn get_open_position(&self, asset_symbol: String) -> Result<GetOpenPositionResponse> {
        let position = self
            .apca_client
            .issue::<position::Get>(&Symbol::Sym(asset_symbol))
            .await?;

        Ok(GetOpenPositionResponse {
            open_position: position.into(),
        })
    }
}

// Tests use environment variable keys for api secret, so make sure those are set to a paper test account
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::{Amount, AssetPair};
    use crate::api::provider::IronTradeClientProvider;
    use crate::api::response::OrderStatus;
    use crate::util::alpaca::AlpacaProvider;
    use apca::ApiInfo;
    use num_decimal::Num;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn buy_market_returns_order_id() {
        let mut client = create_client();
        let order_id = client
            .buy_market(MarketOrderRequest {
                asset_pair: AssetPair::from_str("BTC/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from(20),
                },
            })
            .await
            .unwrap()
            .order_id;

        assert_ne!(order_id, "")
    }

    #[tokio::test]
    #[ignore] // can take a while before the buy order is filled in order to verify a successful sale
    async fn sell_market_returns_order_id() {
        let mut client = create_client();

        let buy_order_id = client
            .buy_market(MarketOrderRequest {
                asset_pair: AssetPair::from_str("AAVE/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from(20),
                },
            })
            .await
            .unwrap()
            .order_id;

        loop {
            let orders = client.get_orders().await.unwrap().orders;
            let buy_order = orders
                .iter()
                .find(|order| order.order_id == buy_order_id)
                .unwrap();
            if matches!(buy_order.status, OrderStatus::Filled) {
                break;
            }
            sleep(Duration::from_secs(1)).await;
        }

        let order_id = client
            .sell_market(MarketOrderRequest {
                asset_pair: AssetPair::from_str("AAVE/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap()
            .order_id;

        assert_ne!(order_id, "")
    }

    // TODO: Run this test atomically
    #[tokio::test]
    async fn get_orders() {
        let mut client = create_client();
        let pre_existing_orders = client.get_orders().await.unwrap().orders;

        client
            .buy_market(MarketOrderRequest {
                asset_pair: AssetPair::from_str("BTC/USD").unwrap(),
                amount: Amount::Notional {
                    notional: Num::from(20),
                },
            })
            .await
            .unwrap();

        let orders = client.get_orders().await.unwrap().orders;

        assert!(orders.len() > pre_existing_orders.len())
    }

    #[tokio::test]
    #[ignore] // Only run when it's guaranteed that the paper account has cash
    async fn get_cash() {
        let client = create_client();
        let cash = client.get_cash().await.unwrap().cash;
        assert!(cash > Num::from(0))
    }

    #[tokio::test]
    #[ignore] // Only run when it's guaranteed that the paper account has an open position
    // TODO: place an order instead and remove ignore macro once https://github.com/alpacahq/Alpaca-API/issues/278 is fixed
    async fn get_open_position() {
        let client = create_client();
        let position = client
            .get_open_position("AAVEUSD".into())
            .await
            .unwrap()
            .open_position;
        assert_eq!(position.asset_symbol, "AAVEUSD")
    }

    fn create_client() -> AlpacaClient {
        let api_info = ApiInfo::from_env().unwrap();
        assert!(
            api_info.api_base_url.to_string().contains("paper"),
            "Use a paper account for unit testing"
        );
        AlpacaProvider::new()
            .create_client(AlpacaClientBuilder::new(ApiInfo::from_env().unwrap()))
            .unwrap()
    }
}
