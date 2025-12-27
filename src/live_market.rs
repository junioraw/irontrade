// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Account, Bar, CryptoPair, Order};
use crate::api::request::OrderRequest;
use crate::api::{Client, Environment, Market};
use anyhow::Result;
use async_trait::async_trait;
use live_market::LiveMarket;

struct LiveEnvironment {
    client: Box<dyn Client + Send + Sync>,
    market: LiveMarket,
}

pub fn create_env<T>(client: T) -> impl Environment
where
    T: Client + Send + Sync + 'static,
{
    LiveEnvironment {
        client: Box::new(client),
        market: LiveMarket,
    }
}

#[async_trait]
impl Client for LiveEnvironment {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String> {
        self.client.place_order(req).await
    }

    async fn get_orders(&mut self) -> Result<Vec<Order>> {
        self.client.get_orders().await
    }

    async fn get_order(&mut self, order_id: &str) -> Result<Order> {
        self.client.get_order(order_id).await
    }

    async fn get_account(&mut self) -> Result<Account> {
        self.client.get_account().await
    }
}

#[async_trait]
impl Market for LiveEnvironment {
    async fn get_latest_minute_bar(&self, crypto_pair: &CryptoPair) -> Result<Option<Bar>> {
        self.market.get_latest_minute_bar(crypto_pair).await
    }
}

impl Environment for LiveEnvironment {}

mod live_market {
    use crate::api::Market;
    use crate::api::common::{Bar, CryptoPair};
    use anyhow::Result;
    use async_trait::async_trait;
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, Utc};
    use reqwest::header::{HeaderMap, HeaderValue};
    use serde::Deserialize;
    use serde::de::DeserializeOwned;
    use serde_this_or_that::as_string;
    use std::collections::HashMap;
    use std::str::FromStr;

    pub struct LiveMarket;

    #[async_trait]
    impl Market for LiveMarket {
        async fn get_latest_minute_bar(&self, crypto_pair: &CryptoPair) -> Result<Option<Bar>> {
            let symbol = crypto_pair.to_string().replace("/", "%2F");
            let url = format!(
                "https://data.alpaca.markets/v1beta3/crypto/eu-1/latest/bars?symbols={symbol}"
            );
            let historical_bars_response: HistoricalBarsResponse = execute_request(&url).await?;
            let bar_response = &historical_bars_response.bars[&crypto_pair.to_string()];
            Ok(Some(Bar {
                low: BigDecimal::from_str(&bar_response.low)?,
                high: BigDecimal::from_str(&bar_response.high)?,
                open: BigDecimal::from_str(&bar_response.open)?,
                close: BigDecimal::from_str(&bar_response.close)?,
                date_time: DateTime::<Utc>::from_str(&bar_response.timestamp)?,
            }))
        }
    }

    async fn execute_request<T>(url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut header_map = HeaderMap::new();
        header_map.insert("accept", HeaderValue::from_str("application/json")?);
        let client = reqwest::ClientBuilder::new()
            .default_headers(header_map)
            .build()?;
        let result = client.get(url).send().await;
        match result {
            Ok(response) => Ok(response.json().await?),
            Err(err) => anyhow::bail!(err),
        }
    }

    #[derive(Deserialize, Debug)]
    struct HistoricalBarsResponse {
        bars: HashMap<String, BarResponse>,
    }

    #[derive(Deserialize, Debug)]
    struct BarResponse {
        #[serde(rename = "o", deserialize_with = "as_string")]
        open: String,

        #[serde(rename = "c", deserialize_with = "as_string")]
        close: String,

        #[serde(rename = "l", deserialize_with = "as_string")]
        low: String,

        #[serde(rename = "h", deserialize_with = "as_string")]
        high: String,

        #[serde(rename = "t")]
        timestamp: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::str::FromStr;

        #[tokio::test]
        async fn get_latest_bar() -> Result<()> {
            let market = LiveMarket;
            let crypto_pair = CryptoPair::from_str("BTC/USD")?;
            let latest_bar = market.get_latest_minute_bar(&crypto_pair).await?;
            assert!(latest_bar.is_some());
            Ok(())
        }
    }
}
