extern crate hex;
extern crate secp256k1;
use flow_rust_sdk::flow::*;
use flow_rust_sdk::{
    build_transaction, check_availability, execute_script, execute_transaction, get_account,
    get_block, get_transaction_result, sign_transaction, Sign,
};
use p256::ecdsa::{signature::Signature, signature::Signer, SigningKey};
use p256::elliptic_curve::SecretKey;
pub extern crate serde_rlp;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn padding(vec: &mut Vec<u8>, count: usize) {
    let mut i: usize = count;
    i = i - vec.len();
    while i > 0 {
        vec.push(0);
        i = i - 1;
    }
}

/// Don't edit this struct, else it will break signing
#[derive(Serialize, Deserialize, Debug)]
struct PayloadCanonicalForm {
    Script: Bytes,
    Arguments: Vec<Bytes>,
    ReferenceBlockID: Bytes,
    GasLimit: u64,
    ProposalKeyAddress: Bytes,
    ProposalKeyIndex: u32,
    ProposalKeySequenceNumber: u64,
    Payer: Bytes,
    Authorizers: Vec<Bytes>,
}

fn payload_from_transaction(transaction: Transaction) -> PayloadCanonicalForm {
    let proposal_key = transaction.proposal_key.unwrap();
    let mut proposal_address = proposal_key.address;
    padding(&mut proposal_address, 8);
    let mut ref_block = transaction.reference_block_id;
    padding(&mut ref_block, 32);
    return PayloadCanonicalForm {
        Script: Bytes::from(transaction.script),
        Arguments: transaction
            .arguments
            .into_iter()
            .map(|x| Bytes::from(x))
            .collect(),
        ReferenceBlockID: Bytes::from(ref_block),
        GasLimit: transaction.gas_limit,
        ProposalKeyAddress: Bytes::from(proposal_address),
        ProposalKeyIndex: proposal_key.key_id,
        ProposalKeySequenceNumber: proposal_key.sequence_number,
        Payer: Bytes::from(transaction.payer),
        Authorizers: transaction
            .authorizers
            .into_iter()
            .map(|x| Bytes::from(x))
            .collect(),
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // check if the node is available
    check_availability("http://localhost:3569".to_string()).await?;

    // get account at address
    let acct: Account = get_account(
        "http://localhost:3569".to_string(),
        "f8d6e0586b0a20c7".to_string(),
    )
    .await?
    .account
    .unwrap();

    // Print out the address and balance of the account
    println!("Address: {:?}", hex::encode(&acct.address));
    println!("Balance: {:?}", &acct.balance);

    // Define script
    let script = b"
    import Crypto

    pub fun main(): Bool {
        let keyList = Crypto.KeyList()
    
        let publicKeyA = PublicKey(
            publicKey:
                \"ef100c2a8d04de602cd59897e08001cf57ca153cb6f9083918cde1ec7de77418a2c236f7899b3f786d08a1b4592735e3a7461c3e933f420cf9babe350abe0c5a\".decodeHex(),
            signatureAlgorithm: SignatureAlgorithm.ECDSA_P256
        )
        keyList.add(
            publicKeyA,
            hashAlgorithm: HashAlgorithm.SHA3_256,
            weight: 1.0
        )
    
        let signatureSet = [
            Crypto.KeyListSignature(
                keyIndex: 0,
                signature:
                    \"12adbf7d71d8ba2febf7922b001a9950248aba40300f23ee9922fafb39979af72ed50b752577d81dfec406151e2ca8bbedd220d9a0bb61b11e7017326c46daca\".decodeHex()
            )
        ]
    
        let signedData = \"68656c6c6f776f726c64\".decodeHex()
    
        let isValid = keyList.verify(
            signatureSet: signatureSet,
            signedData: signedData
        )
        return isValid;
    }";

    // Send script to the blockchain
    let script_results: ExecuteScriptResponse =
        execute_script("http://localhost:3569".to_string(), script.to_vec()).await?;
    let v: Value = serde_json::from_str(&String::from_utf8(script_results.value).unwrap())?;
    println!("Signature is valid: {}", v["value"]);

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
        address: hex::decode("f8d6e0586b0a20c7").unwrap(),
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
        ["f8d6e0586b0a20c7".to_string()].to_vec(),
        "f8d6e0586b0a20c7".to_string(),
    )
    .await?;

    // sign the transaction
    let signature = Sign {
        address: "f8d6e0586b0a20c7".to_owned(),
        key_id: 0,
        private_key: "324db577a741a9b7a2eb6cef4e37e72ff01a554bdbe4bd77ef9afe1cb00d3cec".to_owned(),
    };
    let signed: Option<Transaction> =
        sign_transaction(build, vec![], vec![signature], None).await?;

    // send to the blockchain
    let transaction_execution: SendTransactionResponse =
        execute_transaction("http://localhost:3569".to_string(), signed).await?;

    println!("{:?}", hex::encode(&transaction_execution.id));
    // get the result of the transaction execution
    let get_transaction_result: TransactionResultResponse = get_transaction_result(
        "http://localhost:3569".to_string(),
        transaction_execution.id,
    )
    .await?;

    println!("{:?}", &get_transaction_result);

    // testing signatures
    let secret_bytes = Bytes::from(
        hex::decode("324db577a741a9b7a2eb6cef4e37e72ff01a554bdbe4bd77ef9afe1cb00d3cec").unwrap(),
    );
    let secret_key = SecretKey::from_be_bytes(&secret_bytes).unwrap();
    let sig_key = SigningKey::from(&secret_key);
    let mut domain_tag: Vec<u8> = b"FLOW-V0.0-user".to_vec();
    padding(&mut domain_tag, 32);

    let signature = sig_key.sign(&[domain_tag, b"helloworld".to_vec()].concat());
    println!("{}", hex::encode(&signature.as_bytes()));
    println!("{}", hex::encode(b"helloworld"));

    Ok(())
}
