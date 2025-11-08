// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

pub trait IronTradeClientProvider<T: IronTradeClient, U: AccountConfig> {
    async fn get_client(
        &self,
        account_config: U,
    ) -> Result<T>;
}

pub trait AccountConfig {}