use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn xxfunc_proc(input: ItemFn) -> TokenStream {
    let fn_name = &input.sig.ident;
    let fn_body = &input.block;

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn #fn_name(data_ptr: *const u8, data_size: usize) -> i64 {
            let data = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };
            #fn_body
        }
    };

    expanded
}
