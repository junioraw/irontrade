// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::{DateTime, Utc};
use std::ops::Mul;

pub trait Clock {
    fn now(&mut self) -> DateTime<Utc>;
}

pub struct SpeedClock {
    last_artificial_time: DateTime<Utc>,
    last_actual_time: DateTime<Utc>,
    speed_multiplier: i32,
}

impl SpeedClock {
    pub fn start(start_time: DateTime<Utc>, speed_multiplier: i32) -> Self {
        SpeedClock {
            last_artificial_time: start_time,
            last_actual_time: Utc::now(),
            speed_multiplier,
        }
    }
}

impl Clock for SpeedClock {
    fn now(&mut self) -> DateTime<Utc> {
        let last_actual_time = Utc::now();
        let time_delta = last_actual_time - self.last_actual_time;
        self.last_actual_time = last_actual_time;

        let last_artificial_time =
            self.last_artificial_time + time_delta.mul(self.speed_multiplier);
        self.last_artificial_time = last_artificial_time;

        last_artificial_time
    }
}