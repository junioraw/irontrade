// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::Client;
use crate::api::market::Market;

pub trait Environment {
    fn get_client() -> impl Client;
    fn get_market() -> impl Market;
}