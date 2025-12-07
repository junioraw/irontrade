// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{AssetPair, Bar};
use anyhow::Result;

pub trait Market {
    fn get_latest_minute_bar(
        &self,
        asset_pair: &AssetPair,
    ) -> impl Future<Output = Result<Bar>> + Send;
}

pub trait Watcher {
    fn wait_for_next_bar(&self, asset_pair: &AssetPair)
    -> impl Future<Output = Result<Bar>> + Send;
}
