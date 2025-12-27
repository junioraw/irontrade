// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use async_trait::async_trait;
use crate::api::client::Client;
use crate::api::market::Market;

#[async_trait]
pub trait Environment: Client + Market {}