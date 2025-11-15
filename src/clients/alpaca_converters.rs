// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::common::Amount;
use crate::api::response::{OpenPosition, Order, OrderStatus, OrderType};
use apca::api::v2::order::Amount as ApcaAmount;
use apca::api::v2::order::Order as ApcaOrder;
use apca::api::v2::order::Status as ApcaOrderStatus;
use apca::api::v2::order::Type;
use apca::api::v2::position::Position;

impl From<ApcaAmount> for Amount {
    fn from(amount: ApcaAmount) -> Self {
        match amount {
            ApcaAmount::Quantity { quantity } => Self::Quantity { quantity },
            ApcaAmount::Notional { notional } => Self::Notional { notional },
        }
    }
}

impl From<Amount> for ApcaAmount {
    fn from(amount: Amount) -> Self {
        match amount {
            Amount::Quantity { quantity } => Self::Quantity { quantity },
            Amount::Notional { notional } => Self::Notional { notional },
        }
    }
}

impl From<Position> for OpenPosition {
    fn from(position: Position) -> Self {
        Self {
            asset_symbol: position.symbol.to_string(),
            average_entry_price: position.average_entry_price,
            quantity: position.quantity,
            market_value: position.market_value,
        }
    }
}

impl From<ApcaOrderStatus> for OrderStatus {
    fn from(status: ApcaOrderStatus) -> Self {
        match status {
            ApcaOrderStatus::New => Self::New,
            ApcaOrderStatus::PartiallyFilled => Self::PartiallyFilled,
            ApcaOrderStatus::Filled => Self::Filled,
            ApcaOrderStatus::Expired => Self::Expired,
            _ => Self::Unimplemented,
        }
    }
}

impl From<Type> for OrderType {
    fn from(type_: Type) -> Self {
        match type_ {
            Type::Market => Self::Market,
            Type::Limit => Self::Limit,
            _ => todo!(),
        }
    }
}

impl From<ApcaOrder> for Order {
    fn from(order: ApcaOrder) -> Self {
        Self {
            order_id: order.id.to_string(),
            asset_symbol: order.symbol,
            filled_quantity: order.filled_quantity,
            amount: order.amount.into(),
            average_fill_price: order.average_fill_price,
            status: order.status.into(),
            type_: order.type_.into(),
        }
    }
}
