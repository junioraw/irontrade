// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Bar, CryptoPair};
use anyhow::Result;

pub trait Market {
    fn get_latest_minute_bar(
        &self,
        crypto_pair: &CryptoPair
    ) -> impl Future<Output = Result<Option<Bar>>> + Send;
}
