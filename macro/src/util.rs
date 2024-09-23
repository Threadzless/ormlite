use proc_macro2::TokenStream;
use quote::quote;

pub fn box_fut_ts() -> TokenStream {
    quote!(::ormlite::BoxFuture)
}
