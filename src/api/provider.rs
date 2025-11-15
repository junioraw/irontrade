// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

pub trait IronTradeClientProvider<T: IronTradeClient> {
    fn create_client<U: IronTradeClientBuilder<T>>(&self, builder: U) -> Result<T> {
        builder.build()
    }
}

pub trait IronTradeClientBuilder<T: IronTradeClient> {
    fn build(self) -> Result<T>;
}