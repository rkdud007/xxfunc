use crate::xxfunc_proc::xxfunc_proc;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

mod xxfunc_proc;

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    xxfunc_proc(input).into()
}
