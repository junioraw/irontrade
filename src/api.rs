// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

pub use client::Client;
mod client;

pub mod request;
pub mod common;

pub use market::Market;
mod market;

pub use environment::Environment;
mod environment;
