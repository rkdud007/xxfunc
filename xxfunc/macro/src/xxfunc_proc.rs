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
    let return_type = &input_fn.sig.output;
    let is_async = input_fn.sig.asyncness.is_some();

    let inner_fn = if is_async {
        quote! {
            async fn __xxfunc_inner(data: &[u8]) #return_type {
                #fn_body
            }
        }
    } else {
        quote! {
            fn __xxfunc_inner(data: &[u8]) #return_type {
                #fn_body
            }
        }
    };

    let runtime_creation = if is_async {
        quote! {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            rt.block_on(async { __xxfunc_inner(data).await })
        }
    } else {
        quote! { __xxfunc_inner(data) }
    };

    let expanded = quote! {
        use core::slice;
        use std::sync::Mutex;

        static LATEST_ALLOCATION: Mutex<Option<(u64, u64)>> = Mutex::new(None);

        #[no_mangle]
        pub extern "C" fn alloc(data_size: u64) -> u64 {
            let mut buf = Vec::with_capacity(data_size as usize);
            let ptr = buf.as_mut_ptr();
            std::mem::forget(buf);
            let data_ptr = ptr as *const u8 as u64;

            *LATEST_ALLOCATION.lock().expect("failed to acquire mutex") = Some((ptr as u64, data_size));

            data_ptr
        }

        #[no_mangle]
        pub extern "C" fn process(data_ptr: u64, data_size: u64) -> u64 {
            assert_eq!(
                (data_ptr, data_size),
                LATEST_ALLOCATION.lock().expect("failed to acquire mutex").expect("no last allocation")
            );

            let data = unsafe { slice::from_raw_parts(data_ptr as *const u8, data_size as usize) };

            #runtime_creation;

            let notification = String::from_utf8_lossy(data);
            notification.len() as u64
        }

        #inner_fn

        fn main() {}
    };

    expanded
}
