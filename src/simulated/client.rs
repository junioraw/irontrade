// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::common::{Account, CryptoPair, OpenPosition, Order};
use crate::api::request::OrderRequest;
use crate::simulated::broker::SimulatedBroker;
use anyhow::Result;
use bigdecimal::BigDecimal;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SimulatedClient {
    broker: SimulatedBroker,
}

impl SimulatedClient {
    pub fn new(broker: SimulatedBroker) -> Self {
        Self { broker }
    }
    pub fn set_notional_per_unit(
        &mut self,
        crypto_pair: CryptoPair,
        notional_value_per_unit: BigDecimal,
    ) -> Result<()> {
        self.broker
            .set_notional_value_per_unit(crypto_pair, notional_value_per_unit)
    }
}

impl SimulatedClient {
    fn get_open_position(&self, asset_symbol: &str) -> Result<OpenPosition> {
        let balance = self.broker.get_balance(asset_symbol);
        let notional_per_unit = self.broker.get_notional_per_unit(&CryptoPair {
            notional_coin: self.broker.get_currency(),
            quantity_coin: asset_symbol.into(),
        })?;
        let open_position = OpenPosition {
            asset_symbol: asset_symbol.into(),
            quantity: balance.clone(),
            average_entry_price: None,
            market_value: Some(balance * notional_per_unit),
        };
        Ok(open_position)
    }
}

impl Client for SimulatedClient {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String> {
        let order_id = self.broker.place_order(req)?;
        Ok(order_id)
    }

    async fn get_orders(&mut self) -> Result<Vec<Order>> {
        let orders = self.broker.get_orders();
        Ok(orders)
    }

    async fn get_order(&mut self, order_id: &str) -> Result<Order> {
        let order = self.broker.get_order(order_id)?;
        Ok(order)
    }

    async fn get_account(&mut self) -> Result<Account> {
        let currency = &self.broker.get_currency();
        let mut open_positions = HashMap::new();
        for symbol in self.broker.get_purchased_asset_symbols() {
            let open_position = self.get_open_position(&symbol)?;
            open_positions.insert(symbol, open_position);
        }
        let cash = self.broker.get_balance(currency);
        let buying_power = self.broker.get_buying_power(currency);
        let account = Account {
            open_positions,
            cash,
            buying_power,
            currency: currency.into(),
        };
        Ok(account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::common::{Amount, Order, OrderSide, OrderStatus, OrderType};
    use crate::simulated::broker::SimulatedBrokerBuilder;
    use std::str::FromStr;

    const TEN_DOLLARS_COIN: &str = "TEN";
    const TEN_DOLLARS_CRYPTO_PAIR: &str = "TEN/USD";

    #[tokio::test]
    async fn buy_market_returns_order_id() -> Result<()> {
        let mut client = create_client()?;

        let order_request = OrderRequest::market_buy(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        let order_id = client.place_order(order_request).await?;

        assert_ne!(order_id, "");

        Ok(())
    }

    #[tokio::test]
    async fn sell_market_returns_order_id() -> Result<()> {
        let mut client = create_client()?;

        let buy_request = OrderRequest::market_buy(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        client.place_order(buy_request).await?;

        let sell_request = OrderRequest::market_sell(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );
        let order_id = client.place_order(sell_request).await?;

        assert_ne!(order_id, "");

        Ok(())
    }

    #[tokio::test]
    async fn get_orders_returns_all_placed_orders() -> Result<()> {
        let mut client = create_client()?;

        assert_eq!(client.get_orders().await?.len(), 0);

        let buy_request = OrderRequest::market_buy(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        let buy_order_id = client.place_order(buy_request).await?;

        assert_eq!(client.get_orders().await?.len(), 1);

        let sell_request = OrderRequest::market_sell(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        let sell_order_id = client.place_order(sell_request).await?;

        assert_eq!(client.get_orders().await?.len(), 2);

        let buy_order = client.get_order(&buy_order_id).await?;

        let expected_order = Order {
            order_id: buy_order_id,
            asset_symbol: TEN_DOLLARS_CRYPTO_PAIR.into(),
            amount: Amount::Notional {
                notional: BigDecimal::from(10),
            },
            limit_price: None,
            filled_quantity: BigDecimal::from(1),
            average_fill_price: Some(BigDecimal::from(10)),
            status: OrderStatus::Filled,
            type_: OrderType::Market,
            side: OrderSide::Buy,
        };

        assert_eq!(buy_order, expected_order,);

        let sell_order = client.get_order(&sell_order_id).await?;

        let expected_order = Order {
            order_id: sell_order_id,
            side: OrderSide::Sell,
            ..expected_order
        };

        assert_eq!(sell_order, expected_order,);

        Ok(())
    }

    #[tokio::test]
    async fn get_cash_returns_current_balance() -> Result<()> {
        let mut client = create_client()?;

        assert_eq!(client.get_account().await?.cash, BigDecimal::from(1000));

        let order_request = OrderRequest::market_buy(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        client.place_order(order_request).await?;

        assert_eq!(client.get_account().await?.cash, BigDecimal::from(990));

        let order_request = OrderRequest::market_sell(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(5),
            },
        );
        client.place_order(order_request).await?;

        assert_eq!(client.get_account().await?.cash, BigDecimal::from(995));

        Ok(())
    }

    #[tokio::test]
    async fn get_open_position() -> Result<()> {
        let mut client = create_client()?;

        assert_eq!(
            client
                .get_account()
                .await?
                .open_positions
                .get(TEN_DOLLARS_COIN),
            None
        );

        let order_request = OrderRequest::market_buy(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(15),
            },
        );

        client.place_order(order_request).await?;

        assert_eq!(
            client.get_account().await?.open_positions[TEN_DOLLARS_COIN],
            OpenPosition {
                asset_symbol: TEN_DOLLARS_COIN.into(),
                average_entry_price: None,
                quantity: BigDecimal::from_str("1.5")?,
                market_value: Some(BigDecimal::from(15)),
            }
        );

        let order_request = OrderRequest::market_sell(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            Amount::Notional {
                notional: BigDecimal::from(10),
            },
        );

        client.place_order(order_request).await?;

        assert_eq!(
            client.get_account().await?.open_positions[TEN_DOLLARS_COIN],
            OpenPosition {
                asset_symbol: TEN_DOLLARS_COIN.into(),
                average_entry_price: None,
                quantity: BigDecimal::from_str("0.5")?,
                market_value: Some(BigDecimal::from(5)),
            }
        );

        Ok(())
    }

    fn create_client() -> Result<impl Client> {
        let broker = SimulatedBrokerBuilder::new("USD")
            .set_balance(BigDecimal::from(1000))
            .build();
        let mut client = SimulatedClient::new(broker);
        client.set_notional_per_unit(
            CryptoPair::from_str(TEN_DOLLARS_CRYPTO_PAIR)?,
            BigDecimal::from(10),
        )?;
        Ok(client)
    }
}
