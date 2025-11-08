// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

trait IronTradeClientProvider {
    async fn live_client<T: IronTradeClient>(&self) -> Result<Option<T>>;
    async fn paper_client<T: IronTradeClient>(&self) -> Result<Option<T>>;
}