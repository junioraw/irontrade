// SPDX-License-Identifier: GPL-3.0-or-later
use anyhow::Result;

trait IronTradeClient {
    async fn buy_market(req: BuyMarketRequest) -> Result<BuyMarketResponse>;
    async fn sell_market(req: SellMarketRequest) -> Result<SellMarketRequest>;
    async fn get_orders() -> Result<GetOrdersResponse>;
    async fn get_order(order_id: String) -> Result<GetOrderResponse>;
    async fn get_positions() -> Result<GetPositionsResponse>;
}

struct BuyMarketRequest {}
struct SellMarketRequest {}

struct BuyMarketResponse {}
struct SellMarketResponse {}
struct GetOrderResponse {}
struct GetOrdersResponse {}
struct GetPositionsResponse {}