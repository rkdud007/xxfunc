# example of `#[xxfunc::main]` macro

user code should be in `src/main.rs`

```rust
#![feature(prelude_import)]
#![no_main]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use core::slice;
use std::sync::Mutex;
static LATEST_ALLOCATION: Mutex<Option<(u64, u64)>> = Mutex::new(None);
#[no_mangle]
pub extern "C" fn alloc(data_size: u64) -> u64 {
    let mut buf = Vec::with_capacity(data_size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    let data_ptr = ptr as *const u8 as u64;
    *LATEST_ALLOCATION.lock().expect("failed to acquire mutex") = Some((
        ptr as u64,
        data_size,
    ));
    data_ptr
}
#[no_mangle]
pub extern "C" fn process(data_ptr: u64, data_size: u64) -> u64 {
    match (
        &(data_ptr, data_size),
        &LATEST_ALLOCATION
            .lock()
            .expect("failed to acquire mutex")
            .expect("no last allocation"),
    ) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
    let data = unsafe {
        slice::from_raw_parts(data_ptr as *const u8, data_size as usize)
    };
    __xxfunc_inner(data);
    let notification = String::from_utf8_lossy(data);
    notification.len() as u64
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

Then you can build into wasi binary

```console
cargo xxxfunc build
```
