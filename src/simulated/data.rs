// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use chrono::{DateTime, Utc};
use crate::api::common::{CryptoPair, Bar};

pub trait BarDataSource {
    fn get_bar(&self, asset_pair: &CryptoPair, date_time: &DateTime<Utc>) -> Result<Option<Bar>>;
}