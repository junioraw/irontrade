// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Bar, CryptoPair};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Market {
    async fn get_latest_minute_bar(
        &self,
        crypto_pair: &CryptoPair,
    ) -> Result<Option<Bar>>;
}
