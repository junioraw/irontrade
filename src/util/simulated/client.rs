use crate::api::client::IronTradeClient;
use crate::api::common::{Amount, AssetPair, OpenPosition};
use crate::api::request::MarketOrderRequest;
use crate::api::response::{
    GetCashResponse, GetOpenPositionResponse, GetOrdersResponse, MarketOrderResponse
};
use crate::util::simulated::broker::SimulatedBroker;
use anyhow::Result;
use num_decimal::Num;

pub struct SimulatedClient {
    broker: SimulatedBroker,
}

impl SimulatedClient {
    pub fn new(broker: SimulatedBroker) -> Self {
        Self { broker }
    }
    pub fn set_notional_per_unit(
        &mut self,
        asset_pair: AssetPair,
        notional_per_unit: Num,
    ) -> Result<()> {
        self.broker
            .set_notional_per_unit(asset_pair, notional_per_unit)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::simulated::broker::SimulatedBrokerBuilder;
    use num_decimal::Num;
    use std::str::FromStr;

    const TEN_DOLLARS_COIN: &str = "TEN";
    const TEN_DOLLARS_COIN_PAIR: &str = "TEN/USD";

    #[tokio::test]
    async fn buy_market_returns_order_id() {
        let mut client = create_client();

        let order_id = client
            .buy_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap()
            .order_id;

        assert_ne!(order_id, "");
    }

    #[tokio::test]
    async fn sell_market_returns_order_id() {
        let mut client = create_client();

        client
            .buy_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap();
        let order_id = client
            .sell_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap()
            .order_id;

        assert_ne!(order_id, "");
    }

    #[tokio::test]
    async fn get_orders_returns_all_placed_orders() {
        let mut client = create_client();

        assert_eq!(client.get_orders().await.unwrap().orders.len(), 0);

        client
            .buy_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap();

        assert_eq!(client.get_orders().await.unwrap().orders.len(), 1);

        client
            .sell_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap();

        assert_eq!(client.get_orders().await.unwrap().orders.len(), 2);
    }

    #[tokio::test]
    async fn get_cash_returns_current_balance() {
        let mut client = create_client();

        assert_eq!(client.get_cash().await.unwrap().cash, Num::from(1000));

        client
            .buy_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap();

        assert_eq!(client.get_cash().await.unwrap().cash, Num::from(990));

        client
            .sell_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(5),
                },
            })
            .await
            .unwrap();

        assert_eq!(client.get_cash().await.unwrap().cash, Num::from(995));
    }

    #[tokio::test]
    async fn get_open_position() {
        let mut client = create_client();

        assert_eq!(
            client
                .get_open_position(TEN_DOLLARS_COIN)
                .await
                .unwrap()
                .open_position,
            OpenPosition {
                asset_symbol: TEN_DOLLARS_COIN.into(),
                average_entry_price: None,
                quantity: Num::from(0),
                market_value: Some(Num::from(0)),
            }
        );

        client
            .buy_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(15),
                },
            })
            .await
            .unwrap();

        assert_eq!(
            client
                .get_open_position(TEN_DOLLARS_COIN)
                .await
                .unwrap()
                .open_position,
            OpenPosition {
                asset_symbol: TEN_DOLLARS_COIN.into(),
                average_entry_price: None,
                quantity: Num::from_str("1.5").unwrap(),
                market_value: Some(Num::from(15)),
            }
        );

        client
            .sell_market(MarketOrderRequest {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
            })
            .await
            .unwrap();

        assert_eq!(
            client
                .get_open_position(TEN_DOLLARS_COIN)
                .await
                .unwrap()
                .open_position,
            OpenPosition {
                asset_symbol: TEN_DOLLARS_COIN.into(),
                average_entry_price: None,
                quantity: Num::from_str("0.5").unwrap(),
                market_value: Some(Num::from(5)),
            }
        );
    }

    fn create_client() -> SimulatedClient {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(Num::from(1000))
            .build();
        let mut client = SimulatedClient::new(broker);
        client
            .set_notional_per_unit(ten_dollars_asset_pair(), Num::from(10))
            .unwrap();
        client
    }

    fn ten_dollars_asset_pair() -> AssetPair {
        AssetPair::from_str(TEN_DOLLARS_COIN_PAIR).unwrap()
    }
}
