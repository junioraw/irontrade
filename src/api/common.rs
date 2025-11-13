// SPDX-License-Identifier: GPL-3.0-or-later

use num_decimal::Num;

pub enum Amount {
    Quantity { quantity: Num },
    Notional { notional: Num },
}