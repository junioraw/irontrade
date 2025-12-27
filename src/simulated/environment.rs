// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::Client;
use crate::api::Environment;
use crate::api::Market;
use crate::api::common::{Account, Bar, CryptoPair, Order};
use crate::api::request::OrderRequest;
use crate::simulated::client::SimulatedClient;
use crate::simulated::context::SimulatedContext;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;

/// [Environment] implementation that simulates price changes based on an internal clock, created by the caller and passed via a [SimulatedContext].
/// The prices are set according to the average of the [Bar]'s low high at the "current" point in time.
/// THe "current" [Bar] is used for the effect price of a symbol, while the market's latest bar is set to the last non overlapping [Bar], since in a real world scenario there isn't a current minute aggregated [Bar].
pub struct SimulatedEnvironment {
    context: SimulatedContext,
    client: SimulatedClient,
    last_processed_time: Option<DateTime<Utc>>,
    crypto_pairs_to_trade: HashSet<CryptoPair>,
    bar_duration: Duration,
    refresh_duration: Duration,
}

pub struct SimulatedEnvironmentBuilder {
    context: SimulatedContext,
    client: SimulatedClient,
    crypto_pairs_to_trade: HashSet<CryptoPair>,
    bar_duration: Duration,
    refresh_duration: Duration,
}

impl SimulatedEnvironmentBuilder {
    pub fn new(context: SimulatedContext, client: SimulatedClient) -> Self {
        Self {
            context,
            client,
            crypto_pairs_to_trade: HashSet::new(),
            bar_duration: Duration::minutes(1),
            refresh_duration: Duration::seconds(30),
        }
    }

    pub fn set_crypto_pairs_to_trade(
        &mut self,
        crypto_pairs_to_trade: HashSet<CryptoPair>,
    ) -> &mut Self {
        self.crypto_pairs_to_trade = crypto_pairs_to_trade;
        self
    }

    pub fn set_bar_duration(&mut self, bar_duration: Duration) -> &mut Self {
        self.bar_duration = bar_duration;
        self
    }

    pub fn set_refresh_duration(&mut self, refresh_duration: Duration) -> &mut Self {
        self.refresh_duration = refresh_duration;
        self
    }

    pub fn build(&self) -> SimulatedEnvironment {
        SimulatedEnvironment::new(
            self.context.clone(),
            self.client.clone(),
            self.crypto_pairs_to_trade.clone(),
            self.bar_duration,
            self.refresh_duration,
        )
    }
}

impl SimulatedEnvironment {
    fn new(
        context: SimulatedContext,
        client: SimulatedClient,
        crypto_pairs_to_trade: HashSet<CryptoPair>,
        bar_duration: Duration,
        refresh_duration: Duration,
    ) -> Self {
        SimulatedEnvironment {
            context,
            client,
            last_processed_time: None,
            crypto_pairs_to_trade,
            bar_duration,
            refresh_duration,
        }
    }

    /// Must be called once after the environment has been created and before any [Client] method call.
    pub fn init(&mut self) -> Result<()> {
        if self.last_processed_time.is_some() {
            return Err(anyhow!("Environment has already been initialized"));
        }
        self.last_processed_time = Some(self.context.clock().now());
        self.update()
    }

    fn update(&mut self) -> Result<()> {
        if self.last_processed_time.is_none() {
            return Err(anyhow!("Environment has not been initialized"));
        }
        let now = self.context.clock().now();
        let mut last_processed_time = self.last_processed_time.unwrap_or(now);
        while last_processed_time <= now {
            for crypto_pair in self.crypto_pairs_to_trade.clone() {
                let bar = self.context.bar_data_source().get_bar(
                    &crypto_pair,
                    &now,
                    self.bar_duration,
                )?;
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

#[async_trait]
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

#[async_trait]
impl Market for SimulatedEnvironment {
    async fn get_latest_minute_bar(&self, crypto_pair: &CryptoPair) -> Result<Option<Bar>> {
        let now = self.context.clock().now();
        let bar_duration = Duration::minutes(1);
        let bar = self
            .context
            .bar_data_source()
            .get_bar(crypto_pair, &now, bar_duration)?;
        if bar.is_none() {
            return Ok(None);
        }
        let bar = bar.unwrap();
        if bar.date_time + bar_duration > now {
            // In a real environment bars would only be returned for the past
            return self.context.bar_data_source().get_bar(
                &crypto_pair,
                &(now - bar_duration),
                bar_duration,
            );
        }
        Ok(Some(bar))
    }
}

impl Environment for SimulatedEnvironment {}

#[cfg(test)]
mod tests {
    use crate::api::Client;
    use crate::api::Market;
    use crate::api::common::{Amount, Bar, CryptoPair, OrderStatus};
    use crate::api::request::OrderRequest;
    use crate::simulated::broker::SimulatedBrokerBuilder;
    use crate::simulated::client::SimulatedClient;
    use crate::simulated::context::SimulatedContext;
    use crate::simulated::data::BarDataSource;
    use crate::simulated::environment::{SimulatedEnvironment, SimulatedEnvironmentBuilder};
    use crate::simulated::time::Clock;
    use anyhow::Result;
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, Duration, Utc};
    use std::collections::HashSet;
    use std::str::FromStr;
    use std::sync::{Arc, RwLock};

    #[test]
    fn init_twice() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock, HashSet::new());
        env.init()?;
        let err = env.init().unwrap_err();
        assert_eq!(err.to_string(), "Environment has already been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn place_order_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock, HashSet::new());
        let err = env
            .place_order(OrderRequest::market_buy(
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
    async fn place_order_without_bars() -> Result<()> {
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let data_source = create_data_source(vec![bar_from_three_minutes_ago]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time - Duration::minutes(5),
            added_duration: added_duration.clone(),
        };
        let mut env = create_environment(data_source, clock, HashSet::new());
        env.init()?;

        let result = env
            .place_order(OrderRequest::market_buy(
                "COIN/GBP".parse()?,
                Amount::Quantity {
                    quantity: BigDecimal::from(10),
                },
            ))
            .await;
        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn place_market_order_after_updating_to_current_time() -> Result<()> {
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let data_source = create_data_source(vec![bar_from_three_minutes_ago]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time - Duration::minutes(5),
            added_duration: added_duration.clone(),
        };
        let mut pairs_to_trade = HashSet::new();
        pairs_to_trade.insert(CryptoPair::from_str("COIN/GBP")?);
        let mut env = create_environment(data_source, clock, pairs_to_trade);
        env.init()?;
        *added_duration.write().unwrap() += Duration::minutes(5);
        env.update()?;

        let order_id = env
            .place_order(OrderRequest::market_buy(
                "COIN/GBP".parse()?,
                Amount::Quantity {
                    quantity: BigDecimal::from(10),
                },
            ))
            .await?;
        assert_ne!(order_id, "");

        Ok(())
    }

    #[tokio::test]
    async fn place_limit_order() -> Result<()> {
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let bar_from_two_minutes_ago = create_bar(5, 10, current_time - Duration::minutes(2));
        let data_source =
            create_data_source(vec![bar_from_three_minutes_ago, bar_from_two_minutes_ago]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time - Duration::minutes(5),
            added_duration: added_duration.clone(),
        };
        let mut pairs_to_trade = HashSet::new();
        pairs_to_trade.insert(CryptoPair::from_str("COIN/GBP")?);
        let mut env = create_environment(data_source, clock, pairs_to_trade);
        env.init()?;
        *added_duration.write().unwrap() += Duration::minutes(2);
        env.update()?;

        let order_id = env
            .place_order(OrderRequest::limit_buy(
                "COIN/GBP".parse()?,
                Amount::Quantity {
                    quantity: BigDecimal::from(10),
                },
                BigDecimal::from(9),
            ))
            .await?;
        assert_eq!(env.get_order(&order_id).await?.status, OrderStatus::New);

        *added_duration.write().unwrap() += Duration::minutes(2);
        assert_eq!(env.get_order(&order_id).await?.status, OrderStatus::Filled);

        Ok(())
    }

    #[tokio::test]
    async fn get_orders_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock, HashSet::new());
        let err = env.get_orders().await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_order_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock, HashSet::new());
        let err = env.get_order("123").await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_account_without_init() -> Result<()> {
        let mut env = create_environment(TestDataSource, TestClock, HashSet::new());
        let err = env.get_account().await.unwrap_err();
        assert_eq!(err.to_string(), "Environment has not been initialized");
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_bar_current_time() -> Result<()> {
        let crypto_pair = CryptoPair::from_str("COIN/GBP")?;
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let data_source = create_data_source(vec![bar_from_three_minutes_ago.clone()]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time,
            added_duration: added_duration.clone(),
        };
        let mut env = create_environment(data_source, clock, HashSet::new());
        env.init()?;

        assert_eq!(
            env.get_latest_minute_bar(&crypto_pair).await?,
            Some(bar_from_three_minutes_ago)
        );

        Ok(())
    }

    #[tokio::test]
    async fn get_latest_bar_no_bars_yet_at_clock_time() -> Result<()> {
        let crypto_pair = CryptoPair::from_str("COIN/GBP")?;
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let data_source = create_data_source(vec![bar_from_three_minutes_ago]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time - Duration::minutes(5),
            added_duration: added_duration.clone(),
        };
        let mut env = create_environment(data_source, clock, HashSet::new());
        env.init()?;

        *added_duration.write().unwrap() += Duration::minutes(1) + Duration::seconds(59);
        assert_eq!(env.get_latest_minute_bar(&crypto_pair).await?, None);

        Ok(())
    }

    #[tokio::test]
    async fn get_latest_bar_overlapping_bar() -> Result<()> {
        let crypto_pair = CryptoPair::from_str("COIN/GBP")?;
        let current_time = DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00")?;
        let bar_from_three_minutes_ago = create_bar(10, 20, current_time - Duration::minutes(3));
        let bar_from_two_minutes_ago = create_bar(100, 200, current_time - Duration::minutes(2));
        let data_source = create_data_source(vec![
            bar_from_three_minutes_ago.clone(),
            bar_from_two_minutes_ago,
        ]);
        let added_duration = Arc::new(RwLock::new(Duration::zero()));
        let clock = StepClock {
            initial_time: current_time - Duration::minutes(5),
            added_duration: added_duration.clone(),
        };
        let mut env = create_environment(data_source, clock, HashSet::new());
        env.init()?;

        *added_duration.write().unwrap() += Duration::minutes(3) + Duration::seconds(59);
        assert_eq!(
            env.get_latest_minute_bar(&crypto_pair).await?,
            Some(bar_from_three_minutes_ago)
        );

        Ok(())
    }

    fn create_data_source(ordered_bars: Vec<Bar>) -> impl BarDataSource {
        #[derive(Clone)]
        struct DataSource {
            ordered_bars: Vec<Bar>,
        }
        let data_source = DataSource { ordered_bars };
        impl BarDataSource for DataSource {
            fn get_bar(
                &self,
                _crypto_pair: &CryptoPair,
                date_time: &DateTime<Utc>,
                _bar_duration: Duration,
            ) -> Result<Option<Bar>> {
                for bar in self.ordered_bars.iter().rev() {
                    if bar.date_time <= *date_time {
                        return Ok(Some(bar.clone()));
                    }
                }
                Ok(None)
            }
        }
        data_source
    }

    fn create_bar(low: i32, high: i32, date_time: DateTime<Utc>) -> Bar {
        Bar {
            low: BigDecimal::from(low),
            high: BigDecimal::from(high),
            open: BigDecimal::from(low),
            close: BigDecimal::from(high),
            date_time,
        }
    }

    fn create_environment<B, C>(
        data_source: B,
        clock: C,
        pairs_to_trade: HashSet<CryptoPair>,
    ) -> SimulatedEnvironment
    where
        B: BarDataSource + Send + Sync + 'static,
        C: Clock + Send + Sync + 'static,
    {
        SimulatedEnvironmentBuilder::new(
            SimulatedContext::new(data_source, clock),
            SimulatedClient::new(
                SimulatedBrokerBuilder::new("GBP")
                    .set_balance(BigDecimal::from(100_000))
                    .build(),
            ),
        )
        .set_crypto_pairs_to_trade(pairs_to_trade)
        .set_bar_duration(Duration::minutes(1))
        .set_refresh_duration(Duration::seconds(30))
        .build()
    }

    #[derive(Clone)]
    struct StepClock {
        initial_time: DateTime<Utc>,
        added_duration: Arc<RwLock<Duration>>,
    }

    impl Clock for StepClock {
        fn now(&self) -> DateTime<Utc> {
            self.initial_time + *self.added_duration.read().unwrap()
        }
    }

    #[derive(Clone)]
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

    #[derive(Clone)]
    struct TestClock;

    impl Clock for TestClock {
        fn now(&self) -> DateTime<Utc> {
            DateTime::<Utc>::from_str("2025-12-17T18:30:00+00:00").unwrap()
        }
    }
}
