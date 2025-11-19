// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Amount, AssetPair};

pub struct OrderRequest {
    pub asset_pair: AssetPair,
    pub amount: Amount,
}

