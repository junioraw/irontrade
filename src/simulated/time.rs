// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use dyn_clone::DynClone;

pub trait Clock: DynClone {
    fn now(&self) -> DateTime<Utc>;
}

dyn_clone::clone_trait_object!(Clock);