// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

pub trait IronTradeClientProvider<T: IronTradeClient> {
    fn create_client(&self, builder: impl IronTradeClientBuilder<T>) -> Result<T>;
}

pub trait IronTradeClientBuilder<T: IronTradeClient> {
    fn build(self) -> Result<T>;
}

pub mod simple {
    use crate::api::client::IronTradeClient;
    use crate::provider::{IronTradeClientBuilder, IronTradeClientProvider};

    pub struct SimpleProvider;

    impl<T: IronTradeClient> IronTradeClientProvider<T> for SimpleProvider {
        fn create_client(&self, builder: impl IronTradeClientBuilder<T>) -> anyhow::Result<T> {
            builder.build()
        }
    }
}