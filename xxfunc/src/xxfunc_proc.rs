use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, ItemFn};

pub fn xxfunc_proc(input_fn: ItemFn) -> TokenStream {
    if input_fn.sig.ident != "main" {
        return Error::new_spanned(
            &input_fn.sig.ident,
            "The xxfunc::main attribute can only be used with the main function",
        )
        .to_compile_error();
    }

    let fn_body = &input_fn.block;

    let expanded = quote! {
        use core::slice;
        use std::sync::Mutex;

        static LATEST_ALLOCATION: Mutex<Option<(u64, u64)>> = Mutex::new(None);

        #[no_mangle]
        pub extern "C" fn alloc(data_size: u64) -> u64 {
            let mut buf = Vec::with_capacity(data_size as usize);
            let ptr = buf.as_mut_ptr();
            // Prevent the buffer from being dropped.
            std::mem::forget(buf);
            let data_ptr = ptr as *const u8 as u64;

            *LATEST_ALLOCATION.lock().expect("failed to acquire mutex") = Some((ptr as u64, data_size));

            data_ptr
        }

        #[no_mangle]
        pub extern "C" fn process(data_ptr: u64, data_size: u64) -> u64 {
            // Check that the last allocation matches the passed arguments.
            assert_eq!(
                (data_ptr, data_size),
                LATEST_ALLOCATION.lock().expect("failed to acquire mutex").expect("no last allocation")
            );

            // SAFETY: the memory was allocated by the `alloc` and we check it above.
            let data = unsafe { slice::from_raw_parts(data_ptr as *const u8, data_size as usize) };

            __xxfunc_inner(data);
            // It's just a JSON for now, so let's print it as a string.
            let notification = String::from_utf8_lossy(data);

            notification.len() as u64
        }


        fn __xxfunc_inner(data: &[u8]) -> () {
            #fn_body
        }
    };

    expanded
}
