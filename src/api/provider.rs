// SPDX-License-Identifier: GPL-3.0-or-later

use crate::api::client::IronTradeClient;
use anyhow::Result;

/// A generic trait to create [IronTradeClient] impls.
///
/// This interface is useful for implementations that have a global state outside a particular client creation. For example, a custom backend where the data provider is not tied to a given client instance.
///
/// Simple providers that can define the configuration needed for the [IronTradeClient] in the [IronTradeClientBuilder] implementation can implement this method without overriding its implementation of `create_client`
///
/// #Example
///
/// ```
/// use anyhow::Result;
/// use irontrade::api::client::IronTradeClient;
/// use irontrade::api::provider::*;
/// use irontrade::api::request::*;
/// use irontrade::api::response::*;
///
/// let provider = MyClientProvider {};
/// let builder = MyClientBuilder {};
/// let client = provider.create_client(builder).unwrap();
///
/// pub struct MyClientProvider {
///     // My struct variables
/// }
///
/// impl IronTradeClientProvider<MyClient> for MyClientProvider {}
///
/// pub struct MyClient {
///     // My client variables here
/// }
///
/// pub struct MyClientBuilder {
///     // My builder variables
/// }
///  
/// impl IronTradeClientBuilder<MyClient> for MyClientBuilder {
///     fn build(self) -> Result<MyClient> {
///         return Ok(MyClient{})
///     }
/// }
///
/// impl IronTradeClient for MyClient {
///     async fn buy_market(&mut self, req: MarketOrderRequest) -> Result<MarketOrderResponse> {
///         unimplemented!("Example code")
///     }
///
///     async fn sell_market(&mut self, req: MarketOrderRequest) -> Result<MarketOrderResponse> {
///         unimplemented!("Example code")
///     }
///
///     async fn get_orders(&self) -> Result<GetOrdersResponse> {
///         unimplemented!("Example code")
///     }
///
///     async fn get_cash(&self) -> Result<GetCashResponse> {
///         unimplemented!("Example code")
///     }
///
///     async fn get_open_position(&self, asset_symbol: &str) -> Result<GetOpenPositionResponse> {
///         unimplemented!("Example code")
///     }
/// }
/// ```
pub trait IronTradeClientProvider<T: IronTradeClient> {
    fn create_client<U: IronTradeClientBuilder<T>>(&self, builder: U) -> Result<T> {
        builder.build()
    }
}

/// Builder for an [IronTradeClient] impl.
///
/// See [IronTradeClientProvider] for an example on how this can be used to create a client from a provider.
pub trait IronTradeClientBuilder<T: IronTradeClient> {
    fn build(self) -> Result<T>;
}