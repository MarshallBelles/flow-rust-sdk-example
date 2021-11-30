use flow_rust_sdk::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // establish a local connection
    let mut connection = FlowConnection::new("grpc://localhost:3569").await?;

    // script with arguments
    let script = b"
        pub fun main(arg1: UFix64, arg2: UInt64, arg3: String, arg4: String): String {
            return arg1.toString()
            .concat(\" \")
            .concat(arg2.toString())
            .concat(\" \")
            .concat(arg3)
            .concat(\" \")
            .concat(arg4)
        }    
    ";

    // floating point numbers
    let arg1 = Argument::ufix64(12765.123456);
    // Integers
    let arg2 = Argument::uint64(500);
    // when dealing with str
    let arg3 = Argument::str("Hello");
    // if you have a String
    let allocated_string = "World!".to_string();
    let arg4 = Argument::string(allocated_string);

    // encode arguments
    let arguments: Vec<Vec<u8>> = vec![
        arg1.encode(),
        arg2.encode(),
        arg3.encode_str(),
        arg4.encode(),
    ];

    // execute_script(script: Vec<u8>, arguments: Vec<Vec<u8>>, block_height: Option<u64>, block_id: Option<Vec<u8>>)
    let execution = &connection.execute_script(script.to_vec(), arguments, None, None).await?;

    // Parse execution into serde_json Value for convenience
    let result: Value = from_slice(&execution.value)?;
    
    println!("Script result: {}", result["value"]);
    Ok(())
}
