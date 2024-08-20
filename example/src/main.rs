#[xxfunc::main]
async fn main(data: &[u8]) {
    println!("🦀 Hello, world from wasi!, exec notification data length: {:?}", data.len());
    
    // You can now use Tokio's async functions directly
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Waited for 1 second using Tokio!");
}