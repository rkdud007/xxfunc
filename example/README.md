# example

## `#[xxfunc::main]` macro

user code should be in `src/main.rs`

```rust
#![no_main]

#[xxfunc::main]
fn main(data: &[u8]) {
    println!("Hello, world!, data length: {}", data.len());
}
```

it will expand into

```rust

#![feature(prelude_import)]
#![no_main]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use core::slice;
use std::sync::Mutex;
static mut GLOBAL_BUFFER: Vec<u8> = Vec::new();
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    unsafe {
        GLOBAL_BUFFER.clear();
        GLOBAL_BUFFER.reserve(size);
        GLOBAL_BUFFER.as_mut_ptr()
    }
}
#[no_mangle]
pub extern "C" fn process(size: usize) {
    let data = unsafe { &GLOBAL_BUFFER[..size] };
    __xxfunc_inner(data);
}
fn __xxfunc_inner(data: &[u8]) -> () {
    {
        {
            ::std::io::_print(
                format_args!("Hello, world!, data length: {0}\n", data.len()),
            );
        };
    }
}
```

## build

```console
cargo xxxfunc build
```

### deploy

```
cargo-xxfunc deploy --url http://0.0.0.0:3000 --wasm-path ./wasm_output/output.wasm
```

### start

```
cargo-xxfunc start --url http://0.0.0.0:3000 --module-name wasm-exex
```
