irontrade
====

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

**irontrade** is a crate for simulating trading with custom data providers. It also publishes its trading interface to easily plug those strategies into actual trading clients.

Usage
----

The following example creates a simulated trading client with an initial balance of 1000 USD. It then places an order to
buy AAPL worth of 100 USD.

```rust
// Create USD based broker
let mut broker = SimulatedBrokerBuilder::new("USD").build();
let mut client = SimulatedClient::new(broker);

// Set asset price
client.set_notional_per_unit("AAPL", Num::from_str("276.39")?);

// Place market buy order
let order_id = client
    .place_order(OrderRequest {
        asset_pair: AssetPair::from_str("AAPL/USD")?,
        amount: Amount::Notional {
            notional: Num::from(100),
        },
        limit_price: None,
        side: OrderSide::Buy,
    })
    .await?;

println!("Placed order with id {order_id}");
```

[docs-rs]: https://docs.rs/irontrade/latest/irontrade/
