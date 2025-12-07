// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::market::Market;

pub trait Environment: Client + Market {}