# example of `#[xxfunc::main]` macro

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
#[no_mangle]
pub extern "C" fn __xxfunc_main(data_ptr: *const u8, data_size: usize) -> () {
    let data = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };
    __xxfunc_inner(data)
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    __xxfunc_inner(&[]);
}
fn __xxfunc_inner(data: &[u8]) -> () {
    let result = {
        {
            {
                ::std::io::_print(
                    format_args!("Hello, world!, data length: {0}\n", data.len()),
                );
            };
        }
    };
    ()
}
```

Then you can build into wasi binary

```console
cargo xxxfunc build
```
