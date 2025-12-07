// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use crate::api::common::{AssetPair, Bar};

pub struct BarDataSource {}

impl BarDataSource {
    pub fn get_bar(&self, asset_pair: &AssetPair, date_time: &DateTime<Utc>) -> Option<Bar> {
        todo!()
    }
}