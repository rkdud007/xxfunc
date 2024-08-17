#![no_main]

#[xxfunc::main]
async fn main(data: &[u8]) {
    println!("🦀 Hello, world from wasi!, exex notification data length: {}", data.len());
}
