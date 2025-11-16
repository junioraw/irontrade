// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::provider::IronTradeClientProvider;
use crate::util::alpaca::client::AlpacaClient;

pub mod client;
mod convert;

pub struct AlpacaProvider {
    #[doc(hidden)]
    _use_new: (),
}

impl AlpacaProvider {
    pub fn new() -> Self {
        Self { _use_new: () }
    }
}

impl IronTradeClientProvider<AlpacaClient> for AlpacaProvider {}
