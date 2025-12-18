// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{CryptoPair, Bar};
use anyhow::Result;
use chrono::Duration;

pub trait Market {
    fn get_latest_bar(
        &self,
        crypto_pair: &CryptoPair,
        bar_duration: Duration
    ) -> impl Future<Output = Result<Option<Bar>>> + Send;
}
