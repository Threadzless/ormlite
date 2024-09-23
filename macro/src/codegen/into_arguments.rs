use crate::codegen::common::OrmliteCodegen;
use ormlite_attr::TableMeta;
use proc_macro2::TokenStream;
use quote::quote;

/// Allows the model to be turned into arguments. This can be used for bulk insertion.
pub fn impl_IntoArguments(db: &dyn OrmliteCodegen, attr: &TableMeta) -> TokenStream {
    let db = db.database_ts();
    let model = &attr.ident;
    let params = attr.database_columns().map(|c| {
        let field = &c.ident;
        let value = if c.is_json() {
            quote! {
                ::ormlite::types::Json(self.#field)
            }
        } else {
            quote! {
                self.#field
            }
        };
        quote! {
            ::ormlite::Arguments::add(&mut args, #value).unwrap();
        }
    });

    quote! {
        impl<'a> ::ormlite::IntoArguments<'a, #db> for #model {
            fn into_arguments(self) -> <#db as ::ormlite::Database>::Arguments<'a> {
                let mut args = <#db as ::ormlite::Database>::Arguments::<'a>::default();
                #(
                    #params
                )*
                args
            }
        }
    }
}
