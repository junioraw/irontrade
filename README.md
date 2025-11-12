irontrade
====

- [Documentation][docs-rs]
- [Changelog](CHANGELOG.md)

$${\color{red}work \space in \space progress \space - \space DO \space NOT \space USE}$$

**irontrade** is a library for interacting with different trading brokers, as well as creating custom local brokers for more streamlined testing.

For live and paper trading it currently supports Alpaca through [apca](https://github.com/d-e-s-o/apca), which is a large inspiration for this crate.

For testing, it has a simulated trading client provider, which can rely on a remote or local backend.

Example
----

The following example creates a simulated trading client with an initial balance of 1000 USD. It then places an order to buy AAPL worth of 100 USD.

```rust
    // Can be any supported / custom provider and client combination
    let provider = SimulatedIronTradeClientProvider::new(Num::from(1000));
    let mut client = provider.create_client().unwrap();

    let order_id = client
        .buy_market(BuyMarketRequest{
            asset_symbol: "AAPL".into(),
            amount: Amount::Notional {
                notional: Num::from(100)
            }
        })
        .await
        .unwrap()
        .order_id;

    println!("Placed order with id {order_id}");
```

[docs-rs]: https://docs.rs/irontrade/latest/irontrade/
