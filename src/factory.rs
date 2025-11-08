// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use crate::api::client::IronTradeClient;

trait IronTradeFactory {
    async fn default_client<T: IronTradeClient>() -> Result<T>;
    async fn live_client<T: IronTradeClient>() -> Result<Option<T>>;
    async fn paper_client<T: IronTradeClient>() -> Result<Option<T>>;
}
