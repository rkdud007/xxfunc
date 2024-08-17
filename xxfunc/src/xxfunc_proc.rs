use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Error, ItemFn, ReturnType};

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

    let (wasm_return_type, inner_return_type, return_expr) = match return_type {
        ReturnType::Default => (quote!(()), quote!(()), quote!(())),
        ReturnType::Type(_, ty) if ty.to_token_stream().to_string() == "i64" => {
            (quote!(i64), quote!(i64), quote!(result))
        }
        _ => {
            return Error::new_spanned(
                return_type,
                "The main function must either have no return type or return i64",
            )
            .to_compile_error()
        }
    };

    let expanded = quote! {
        #[no_mangle]
        pub extern "C" fn __xxfunc_main(data_ptr: *const u8, data_size: usize) -> #wasm_return_type {
            let data = unsafe { std::slice::from_raw_parts(data_ptr, data_size) };
            __xxfunc_inner(data)
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn main() {
            __xxfunc_inner(&[]);
        }

        fn __xxfunc_inner(data: &[u8]) -> #inner_return_type {
            let result = {
                #fn_body
            };
            #return_expr
        }
    };

    expanded
}
