// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{CryptoPair, Bar};
use anyhow::Result;

pub trait Market {
    fn get_latest_bar(
        &self,
        crypto_pair: &CryptoPair,
    ) -> impl Future<Output = Result<Option<Bar>>> + Send;
}
