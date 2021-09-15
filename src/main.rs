use flow_rust_sdk::flow::*;
use flow_rust_sdk::{
    build_transaction, check_availability, execute_script, execute_transaction, get_account,
    get_block, get_transaction_result, sign_transaction, Sign,
};
use hex;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // check if the node is available
    check_availability("http://localhost:3569".to_string()).await?;

    // get account at address
    let acct: Account = get_account(
        "http://localhost:3569".to_string(),
        "01cf0e2f2f715450".to_string(),
    )
    .await?
    .account
    .unwrap();

    // Print out the address and balance of the account
    println!("Address: {:?}", hex::encode(&acct.address));
    println!("Balance: {:?}", &acct.balance);

    // Define script
    let script = b"
        pub fun main(): String {
            return \"Hello World On Flow!\"
        }";

    // Send script to the blockchain
    let script_results: ExecuteScriptResponse =
        execute_script("http://localhost:3569".to_string(), script.to_vec()).await?;
    let v: Value = serde_json::from_str(&String::from_utf8(script_results.value).unwrap())?;
    println!("{}", v["value"]);

    // define transaction, such as to create a new account
    let transaction = b"
    transaction() {
        prepare(signer: AuthAccount) {
            let acct = AuthAccount(payer: signer)
        }
    }";

    // get the latest block for our transaction request
    let latest_block: BlockResponse =
        get_block("http://localhost:3569".to_string(), None, None, Some(false)).await?;

    // setup proposer
    let proposal_key: TransactionProposalKey = TransactionProposalKey {
        address: hex::decode("01cf0e2f2f715450").unwrap(),
        key_id: 0,
        sequence_number: 0,
    };

    let latest_block_id = latest_block.block.unwrap().id;

    // build the transaction
    let build: Transaction = build_transaction(
        transaction.to_vec(),
        vec![],
        latest_block_id,
        1000,
        proposal_key,
        ["01cf0e2f2f715450".to_string()].to_vec(),
        "01cf0e2f2f715450".to_string(),
    )
    .await?;

    // sign the transaction
    let signature = Sign {
        address: "01cf0e2f2f715450".to_owned(),
        key_id: 0,
        private_key: "3ab30097b08a8ee26014ba3e606f1300757232a116500d807c8d1dfc81d393d5".to_owned(),
    };
    let signature1 = Sign {
        address: "01cf0e2f2f715450".to_owned(),
        key_id: 0,
        private_key: "3ab30097b08a8ee26014ba3e606f1300757232a116500d807c8d1dfc81d393d5".to_owned(),
    };
    let signed: Option<Transaction> = sign_transaction(build, vec![], vec![signature], None).await?;

    // send to the blockchain
    let transaction_execution: SendTransactionResponse =
        execute_transaction("http://localhost:3569".to_string(), signed).await?;

    // get the result of the transaction execution
    let get_transaction_result: TransactionResultResponse = get_transaction_result(
        "http://localhost:3569".to_string(),
        transaction_execution.id,
    )
    .await?;

    println!("{:?}", &get_transaction_result);
    Ok(())
}
