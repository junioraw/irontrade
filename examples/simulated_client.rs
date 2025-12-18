// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use bigdecimal::BigDecimal;
use irontrade::api::client::Client;
use irontrade::api::common::{Amount, CryptoPair, OrderStatus};
use irontrade::api::request::OrderRequest;
use irontrade::simulated::broker::SimulatedBrokerBuilder;
use irontrade::simulated::client::SimulatedClient;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a broker with a balance of 100 GBP and a fee of 0.25% per transaction
    let broker = SimulatedBrokerBuilder::new("GBP")
        .set_fee_percentage_up_to_one_hundred(BigDecimal::from_str("0.25")?)?
        .set_balance(BigDecimal::from(100))
        .build();

    // Create a simulated client using the simulated broker
    let mut client = SimulatedClient::new(broker);

    let avax_gbp_pair = CryptoPair::from_str("AVAX/GBP")?;

    // Set the price of AVAX
    client.set_notional_per_unit(avax_gbp_pair.clone(), BigDecimal::from_str("8.81")?)?;

    let order_id = client
        .place_order(OrderRequest::create_market_buy(
            avax_gbp_pair,
            Amount::Quantity {
                quantity: BigDecimal::from(10),
            },
        ))
        .await?;

    println!("Placed order with id: {order_id}");

    // Retrieve the order status, it should be filled immediately since this is a market buy in the simulated client
    let order_status = client.get_order(&order_id).await?.status;
    assert_eq!(order_status, OrderStatus::Filled);

    // Print balances after transaction
    let account = client.get_account().await?;
    println!("GBP balance: {}", account.cash);

    // AVAX quantity will be smaller than 10 since the broker was built with a 0.25% fee
    println!("AVAX quantity: {}", account.open_positions["AVAX"].quantity);

    Ok(())
}
