// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

trait IronTradeClientProvider {
    async fn get_client<T: IronTradeClient, U: AccountConfig>(&self, account_config: U) -> Result<Option<T>>;
}

trait AccountConfig {}