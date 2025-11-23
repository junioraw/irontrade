use crate::api::client::IronTradeClient;
use crate::api::common::{Amount, AssetPair, OpenPosition};
use crate::api::request::OrderRequestV1;
use crate::api::response::{
    GetCashResponse, GetOpenPositionResponse, GetOrdersResponseV1, OrderResponse,
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
    async fn buy(&mut self, req: OrderRequestV1) -> Result<OrderResponse> {
        let order_id = self.broker.place_order(req)?;
        Ok(OrderResponse { order_id })
    }

    async fn sell(&mut self, req: OrderRequestV1) -> Result<OrderResponse> {
        let req = OrderRequestV1 {
            asset_pair: req.asset_pair,
            amount: match req.amount {
                Amount::Quantity { quantity } => Amount::Quantity {
                    quantity: -quantity,
                },
                Amount::Notional { notional } => Amount::Notional {
                    notional: -notional,
                },
            },
            limit_price: None,
        };
        let order_id = self.broker.place_order(req)?;
        Ok(OrderResponse { order_id })
    }

    async fn get_orders(&self) -> Result<GetOrdersResponseV1> {
        Ok(GetOrdersResponseV1 {
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
    use crate::api::common::{OrderV1, OrderStatus, OrderType};
    use crate::util::simulated::broker::SimulatedBrokerBuilder;
    use num_decimal::Num;
    use std::str::FromStr;

    const TEN_DOLLARS_COIN: &str = "TEN";
    const TEN_DOLLARS_COIN_PAIR: &str = "TEN/USD";

    #[tokio::test]
    async fn buy_market_returns_order_id() {
        let mut client = create_client();

        let order_id = client
            .buy(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
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
            .buy(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
            })
            .await
            .unwrap();
        let order_id = client
            .sell(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
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

        let buy_order_id = client
            .buy(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
            })
            .await
            .unwrap()
            .order_id;

        assert_eq!(client.get_orders().await.unwrap().orders.len(), 1);

        let sell_order_id = client
            .sell(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
            })
            .await
            .unwrap()
            .order_id;

        assert_eq!(client.get_orders().await.unwrap().orders.len(), 2);

        assert_eq!(
            client
                .get_orders()
                .await
                .unwrap()
                .orders
                .iter()
                .filter(|order| order.order_id == buy_order_id)
                .map(OrderV1::clone)
                .last()
                .unwrap(),
            OrderV1 {
                order_id: buy_order_id,
                asset_symbol: TEN_DOLLARS_COIN_PAIR.into(),
                amount: Amount::Quantity {
                    quantity: Num::from(1),
                },
                filled_quantity: Num::from(1),
                average_fill_price: Some(Num::from(10)),
                status: OrderStatus::Filled,
                type_: OrderType::Market,
            }
        );

        assert_eq!(
            client
                .get_orders()
                .await
                .unwrap()
                .orders
                .iter()
                .filter(|order| order.order_id == sell_order_id)
                .map(OrderV1::clone)
                .last()
                .unwrap(),
            OrderV1 {
                order_id: sell_order_id,
                asset_symbol: TEN_DOLLARS_COIN_PAIR.into(),
                amount: Amount::Quantity {
                    quantity: -Num::from(1), // TODO: remove minus sign #14
                },
                filled_quantity: -Num::from(1), // TODO: remove minus sign #14
                average_fill_price: Some(Num::from(10)),
                status: OrderStatus::Filled,
                type_: OrderType::Market,
            }
        );
    }

    #[tokio::test]
    async fn get_cash_returns_current_balance() {
        let mut client = create_client();

        assert_eq!(client.get_cash().await.unwrap().cash, Num::from(1000));

        client
            .buy(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
            })
            .await
            .unwrap();

        assert_eq!(client.get_cash().await.unwrap().cash, Num::from(990));

        client
            .sell(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(5),
                },
                limit_price: None,
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
            .buy(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(15),
                },
                limit_price: None,
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
            .sell(OrderRequestV1 {
                asset_pair: ten_dollars_asset_pair(),
                amount: Amount::Notional {
                    notional: Num::from(10),
                },
                limit_price: None,
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
