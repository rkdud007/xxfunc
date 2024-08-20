#[xxfunc::main]
fn main(data: &[u8]) {
    println!("🦀 Hello, world from wasi!, exec notification data length: {:?}", data.len());
}
