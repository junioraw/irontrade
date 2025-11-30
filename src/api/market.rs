// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{AssetPair, Bar, Quote};
use anyhow::Result;

pub trait Market {
    fn get_latest_minute_bar(
        &self,
        asset_pair: &AssetPair,
    ) -> impl Future<Output = Result<Bar>> + Send;
    fn get_latest_quotes(
        &self,
        asset_pair: &AssetPair,
    ) -> impl Future<Output = Result<Quote>> + Send;
}
