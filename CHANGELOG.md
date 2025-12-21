0.6.0
----
- Update documentation example to match latest API
- Remove `create` prefix in `OrderRequest` creation functions
- Implement fee in `SimulatedClient`
- Rename `AssetPair` to `CryptoPair`
- Implement clock update logic in `SimulatedEnvironment`
- Expose `crate::simulated::time` and `crate::simulated::data` as public APIs
- Replace `num_decimal` with `bigdecimal` for numerical representation
- Add `SimulatedEnvironment`
- Make `BarDataSource` a trait instead of a struct
- Remove `Market.get_latest_quotes`

0.5.1
----
- Make mod `api::environment` public

0.5.0
----
- Rename `IronTradeClient` to `Client`
- Add `api::market::Market` interface
- Add `api::environment::Environment` interface
- Add `Client.get_order(order_id)` function
- Add `api::common::Account` struct and `Client.get_account` function
- Remove `Client.get_buying_power`, `Client.get_cash` and `Client.get_open_position` as those fields are all now part of `Account struct`

0.4.0
----

- Remove `api::response`. These structs were unnecessary as each of them only held a single variable. `IronTradeClient`
  updated accordingly
- Add `IronTradeClient.get_buying_power`

0.3.0
----

- Rename `irontrade::util::simulated` to `irontrade::simulated`

0.2.0
----

- Replace `Client.buy_market`, and `Client.sell_market` with `Client.place_order` which now also supports limit orders
- Add `limit_price` to `Order` and `OrderRequest`
- Add `OrderSide` to `Order` and `OrderRequest`
- Add limit order support to `SimulatedClient`
- Move `OpenPosition`, `Order`, `OrderStatus` and `OrderType` to `api::common`
- Add `OrderSide` to `api::common`
- Copy `README.md` documentation to top level crate documentation

0.1.10
----

- Add `set_notional_per_unit` to `SimulatedClient`

0.1.9
----

- Scrub provider
- Move alpaca client into irontrade_alpaca crate

0.1.8
----

- Documentation for client, common and provider modules.

0.1.7
----

- Make mod alpaca::client public
- Another code formatting fix in README.md

0.1.6
----

- Fix code snippet formatting in README.md

0.1.5
----

- Enable alpaca by default so documentation in docs.rs includes its module

0.1.4
----

- MVP reached for API: implemented all basic methods for existing providers
- Feature flagged the alpaca provider. It is only necessary when callers use this API to interact with Alpaca. The main
  use case for the API is the simulated provider
- Cleaned up and restructured the API. For example order request and response structs were halved given that buy and
  sell orders have the same request and response fields as of now
- Provider and client implementations are now in util package
- API exposes everything an external caller may need to implement their own provider
- IronTradeClientProvider now has a default implementation for create client, which only requires the caller to pass in
  a builder
- Simulated broker now clearly makes a distinction between notional and non-notional assets. It now also supports
  currency (which is enforced to be one of the notional assets)
- Other changes for readability / functionality were done

0.1.3
----

- Added DO NOT USE to README

0.1.2
----

- Added simulated iron trade client
- Added documentation for trait IronTradeClient
- Added README content

0.1.1
-----

- Added AlpacaIronTradeClient with basic support for market buy/sell, orders and positions

0.1.0
-----

- Initial release