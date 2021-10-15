use flow_rust_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = FlowConnection::new("grpc://localhost:3569").await?;

        let payer = "f8d6e0586b0a20c7";
        let payer_private_key = "324db577a741a9b7a2eb6cef4e37e72ff01a554bdbe4bd77ef9afe1cb00d3cec";
        let public_keys = vec!["ef100c2a8d04de602cd59897e08001cf57ca153cb6f9083918cde1ec7de77418a2c236f7899b3f786d08a1b4592735e3a7461c3e933f420cf9babe350abe0c5a".to_owned()];

        let acct = connection.create_account(
            public_keys.to_vec(),
            &payer.to_owned(),
            &payer_private_key.to_owned(),
            0,
        )
        .await
        .expect("Could not create account");
        println!("new account address: {:?}", hex::encode(acct.address));
    Ok(())
}
