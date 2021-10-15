use flow_rust_sdk::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = FlowConnection::new("grpc://localhost:3569").await?;
    
    let script = b"
        pub fun main(arg1: UFix64, arg2: UInt64, arg3: String): UFix64 {
            log(arg2)
            log(arg3)
            return arg1
        }      
    ";
    let arg1 = Argument::ufix64(12765.123456);
    let arg2 = Argument::uint64(500);
    let arg3 = Argument::string("Hello World!".to_string());
    let arguments: Vec<Vec<u8>> = vec![
        arg1.encode(),
        arg2.encode(),
        arg3.encode()
    ];
    println!("{:?}", String::from_utf8(arguments[0].to_vec())?);
    let result: Value = from_slice(&connection.execute_script(script.to_vec(), arguments).await?.value)?;
    println!("Script result: {}", result["value"]);
    Ok(())
}
