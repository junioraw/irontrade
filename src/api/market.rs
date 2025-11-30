// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use crate::api::common::{AssetPair, Bar};

pub trait Market {

    fn get_latest_minute_bar(&self, asset_pair: AssetPair) -> impl Future<Output = Result<Bar>>;
}