// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

pub trait IronTradeClientProvider<T: IronTradeClient> {
    async fn create_client(&self) -> Result<T>;
}
