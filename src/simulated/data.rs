// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::{Bar, CryptoPair};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use dyn_clone::DynClone;

pub trait BarDataSource: DynClone {
    fn get_bar(
        &self,
        crypto_pair: &CryptoPair,
        date_time: &DateTime<Utc>,
        bar_duration: Duration,
    ) -> Result<Option<Bar>>;
}

dyn_clone::clone_trait_object!(BarDataSource);
