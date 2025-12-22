// Copyright (C) 2025 Agostinho Junior
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::simulated::data::BarDataSource;
use crate::simulated::time::Clock;

pub struct SimulatedContext {
    clock: Box<dyn Clock + Send + Sync>,
    bar_data_source: Box<dyn BarDataSource + Send + Sync>,
}

impl SimulatedContext {
    pub fn new<B, C>(bar_data_source: B, clock: C) -> Self
    where
        B: BarDataSource + Send + Sync + 'static,
        C: Clock + Send + Sync + 'static,
    {
        Self { bar_data_source: Box::new(bar_data_source), clock: Box::new(clock) }
    }
    
    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_ref()
    }
    
    pub fn bar_data_source(&self) -> &dyn BarDataSource {
        self.bar_data_source.as_ref()
    }
}
