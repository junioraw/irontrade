// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Duration, Utc};
use std::ops::Mul;

pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

pub struct AutoClock {
    artificial_start_time: DateTime<Utc>,
    actual_start_time: DateTime<Utc>,
    time_speed_multiplier: i32,
}

impl AutoClock {
    pub fn start(start_time: DateTime<Utc>, time_speed_multiplier: i32) -> Self {
        AutoClock {
            artificial_start_time: start_time,
            actual_start_time: Utc::now(),
            time_speed_multiplier,
        }
    }
}

impl Clock for AutoClock {
    fn now(&self) -> DateTime<Utc> {
        let current_time = Utc::now();
        let time_delta = current_time - self.actual_start_time;

        let last_artificial_time =
            self.artificial_start_time + time_delta.mul(self.time_speed_multiplier);

        last_artificial_time
    }
}

pub struct ManualClock {
    artificial_time: DateTime<Utc>,
}

impl ManualClock {
    pub fn new(start_time: DateTime<Utc>) -> Self {
        ManualClock { artificial_time: start_time }
    }

    pub fn advance(&mut self, duration: Duration) {
        self.artificial_time = self.artificial_time + duration;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        self.artificial_time
    }
}