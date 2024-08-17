#![no_main]

#[xxfunc::main]
async fn main(data: &[u8]) {
    println!("Hello, world!, data length: {}", data.len());
}
