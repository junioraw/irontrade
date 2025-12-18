// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::common::{Account, Bar, CryptoPair, Order};
use crate::api::environment::Environment;
use crate::api::market::Market;
use crate::api::request::OrderRequest;
use crate::simulated::client::SimulatedClient;
use crate::simulated::data::BarDataSource;
use crate::simulated::time::Clock;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;

pub struct SimulatedEnvironment {
    client: SimulatedClient,
    bar_data_source: Box<dyn BarDataSource + Send + Sync>,
    last_processed_time: Option<DateTime<Utc>>,
    crypto_pairs_to_trade: HashSet<CryptoPair>,
    clock: Box<dyn Clock + Send + Sync>,
    bar_duration: Duration,
    refresh_duration: Duration,
}

impl SimulatedEnvironment {
    pub fn new<B, C>(
        client: SimulatedClient,
        crypto_pairs_to_trade: HashSet<CryptoPair>,
        bar_data_source: B,
        clock: C,
        bar_duration: Duration,
        refresh_duration: Duration,
    ) -> Self
    where
        B: BarDataSource + Send + Sync + 'static,
        C: Clock + Send + Sync + 'static,
    {
        SimulatedEnvironment {
            client,
            bar_data_source: Box::new(bar_data_source),
            last_processed_time: None,
            crypto_pairs_to_trade,
            clock: Box::new(clock),
            bar_duration,
            refresh_duration,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        if self.last_processed_time.is_some() {
            return Err(anyhow!("Environment has already been initialized"));
        }
        self.last_processed_time = Some(self.clock.now());
        self.update()
    }

    fn update(&mut self) -> Result<()> {
        if self.last_processed_time.is_none() {
            return Err(anyhow!("Environment has not been initialized"));
        }
        let now = self.clock.now();
        let mut last_processed_time = self.last_processed_time.unwrap_or(now);
        while last_processed_time <= now {
            for crypto_pair in self.crypto_pairs_to_trade.clone() {
                let bar = self
                    .bar_data_source
                    .get_bar(&crypto_pair, &now, self.bar_duration)?;
                if let Some(bar) = bar {
                    let value = (bar.low + bar.high) / 2.0;
                    self.client.set_notional_per_unit(crypto_pair, value)?;
                }
            }
            if last_processed_time == now {
                break;
            }
            last_processed_time = DateTime::min(last_processed_time + self.refresh_duration, now);
        }
        self.last_processed_time = Some(now);
        Ok(())
    }
}

impl Client for SimulatedEnvironment {
    async fn place_order(&mut self, req: OrderRequest) -> Result<String> {
        self.update()?;
        self.client.place_order(req).await
    }

    async fn get_orders(&mut self) -> Result<Vec<Order>> {
        self.update()?;
        self.client.get_orders().await
    }

    async fn get_order(&mut self, order_id: &str) -> Result<Order> {
        self.update()?;
        self.client.get_order(order_id).await
    }

    async fn get_account(&mut self) -> Result<Account> {
        self.update()?;
        self.client.get_account().await
    }
}

impl Market for SimulatedEnvironment {
    async fn get_latest_bar(
        &self,
        crypto_pair: &CryptoPair,
        bar_duration: Duration,
    ) -> Result<Option<Bar>> {
        let now = self.clock.now();
        let bar = self
            .bar_data_source
            .get_bar(crypto_pair, &now, bar_duration)?;
        if bar.is_none() {
            return Ok(None);
        }
        let bar = bar.unwrap();
        if bar.date_time + bar_duration > now {
            // In a real environment bars would only be returned for the past
            return Ok(None);
        }
        Ok(Some(bar))
    }
}

impl Environment for SimulatedEnvironment {}

#[cfg(test)]
mod tests {
    use crate::api::client::Client;
    use crate::api::common::{Amount, Bar, CryptoPair};
    use crate::api::request::OrderRequest;
    use crate::simulated::broker::SimulatedBrokerBuilder;
    use crate::simulated::client::SimulatedClient;
    use crate::simulated::data::BarDataSource;
    use crate::simulated::environment::SimulatedEnvironment;
    use crate::simulated::time::Clock;
    use anyhow::Result;
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, Duration, Utc};
    use std::collections::HashSet;
    use std::str::FromStr;

    #[test]
    fn init_twice() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock);
        env.init()?;
        let err = env.init().unwrap_err();
        assert_eq!(err.to_string(), "Environment has already been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn place_order_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock);
        let err = env
            .place_order(OrderRequest::create_market_buy(
                "USDT/GBP".parse()?,
                Amount::Quantity {
                    quantity: BigDecimal::from(10),
                },
            ))
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_orders_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock);
        let err = env.get_orders().await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_order_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock);
        let err = env.get_order("123").await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_account_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock);
        let err = env.get_account().await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    fn create_environment<B, C>(data_source: B, clock: C) -> SimulatedEnvironment
    where
        B: BarDataSource + Send + Sync + 'static,
        C: Clock + Send + Sync + 'static,
    {
        SimulatedEnvironment::new(
            SimulatedClient::new(SimulatedBrokerBuilder::new("GBP").build()),
            HashSet::new(),
            data_source,
            clock,
            Duration::minutes(1),
            Duration::seconds(1),
        )
    }

    struct TestDataSource;
    impl BarDataSource for TestDataSource {
        fn get_bar(
            &self,
            _crypto_pair: &CryptoPair,
            _date_time: &DateTime<Utc>,
            _bar_duration: Duration,
        ) -> Result<Option<Bar>> {
            unimplemented!("Test method")
        }
    }
    struct TestClock;
    impl Clock for TestClock {
        fn now(&self) -> DateTime<Utc> {
            DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00").unwrap()
        }
    }
}
