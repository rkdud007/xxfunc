#![no_main]

#[xxfunc::main]
fn main(data: &[u8]) {
    println!("Hello, world!, data length: {}", data.len());
}
