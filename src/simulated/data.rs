// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Bar, CryptoPair};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

pub trait BarDataSource {
    fn get_bar(
        &self,
        crypto_pair: &CryptoPair,
        date_time: &DateTime<Utc>,
        bar_duration: Duration,
    ) -> Result<Option<Bar>>;
}
