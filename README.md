irontrade
====

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

**irontrade** is a library defining an interface for interacting with different trading brokers, as well as custom data providers for strategy testing.

Usage
----

The following example creates a simulated trading client with an initial balance of 1000 USD. It then places an order to
buy AAPL worth of 100 USD.

```rust
// Create USD based broker
let mut broker = SimulatedBrokerBuilder::new("USD").build();
let mut client = SimulatedClient::new(broker);

let order_id = client
    .buy_market(MarketOrderRequest {
        asset_pair: AssetPair::from_str("AAPL/USD").unwrap(),
        amount: Amount::Notional {
            notional: Num::from(100),
        },
    })
    .await
    .unwrap()
    .order_id;

println!("Placed order with id {order_id}");
```

[docs-rs]: https://docs.rs/irontrade/latest/irontrade/
