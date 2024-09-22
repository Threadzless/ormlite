use crate::codegen::common::OrmliteCodegen;
use crate::codegen::insert::impl_Model__insert;
use crate::codegen::select::impl_Model__select;
use crate::codegen::update::impl_Model__update_all_fields;
use crate::MetadataCache;
use itertools::Itertools;
use ormlite_attr::{ModelMeta, TableMeta};
use proc_macro2::TokenStream;
use quote::quote;

pub fn impl_Model(db: &dyn OrmliteCodegen, attr: &ModelMeta, metadata_cache: &MetadataCache) -> TokenStream {
    let model = &attr.ident;
    let partial_model = attr.builder_struct();

    let impl_Model__insert = impl_Model__insert(db, &attr.table, metadata_cache);
    let impl_Model__update_all_fields = impl_Model__update_all_fields(db, attr);
    let impl_Model__delete = impl_Model__delete(db, attr);
    let impl_Model__fetch_one = impl_Model__fetch_one(db, attr);
    let impl_Model__select = impl_Model__select(db, &attr.table);
    let impl_Model__builder = impl_Model__builder(attr);
    let impl_Model__update_partial = impl_Model__update_partial(attr);
    let impl_Model__create_table = impl_Model__create_table(db, attr);

    let db = db.database_ts();
    quote! {
        impl ::ormlite::model::Model<#db> for #model {
            type ModelBuilder<'a> = #partial_model<'a> where Self: 'a;

            #impl_Model__create_table
            #impl_Model__insert
            #impl_Model__update_all_fields
            #impl_Model__delete
            #impl_Model__fetch_one
            #impl_Model__select

           fn query(query: &str) -> ::ormlite::query::QueryAs<#db, Self, <#db as ::ormlite::Database>::Arguments<'_>> {
                ::ormlite::query_as::<_, Self>(query)
            }

            #impl_Model__builder
            #impl_Model__update_partial

        }
    }
}

pub fn impl_Model__delete(db: &dyn OrmliteCodegen, attr: &ModelMeta) -> TokenStream {
    let mut placeholder = db.placeholder();

    let query = format!(
        "DELETE FROM \"{}\" WHERE {} = {}",
        attr.name,
        attr.pkey.name,
        placeholder.next().unwrap()
    );

    let box_future = crate::util::box_fut_ts();
    let db = db.database_ts();
    let id = &attr.pkey.ident;
    quote! {
        fn delete<'e, E>(self, db: E) -> #box_future<'e, ::ormlite::Result<()>>
        where
            E: 'e +::ormlite::Executor<'e, Database = #db>
        {
            Box::pin(async move {
                let row =::ormlite::query(#query)
                    .bind(self.#id)
                    .execute(db)
                    .await
                    .map_err(::ormlite::Error::from)?;
                if row.rows_affected() == 0 {
                    Err(::ormlite::Error::from(::ormlite::SqlxError::RowNotFound))
                } else {
                    Ok(())
                }
            })
        }
    }
}

pub fn impl_Model__fetch_one(db: &dyn OrmliteCodegen, attr: &ModelMeta) -> TokenStream {
    let mut placeholder = db.placeholder();

    let query = format!(
        "SELECT * FROM \"{}\" WHERE {} = {}",
        attr.name,
        attr.pkey.name,
        placeholder.next().unwrap()
    );

    let db = db.database_ts();
    let box_future = crate::util::box_fut_ts();
    quote! {
        fn fetch_one<'e, 'a, Arg, E>(id: Arg, db: E) -> #box_future<'e, ::ormlite::Result<Self>>
        where
            'a: 'e,
            Arg: 'a + Send + ::ormlite::Encode<'a, #db> + ::ormlite::types::Type<#db>,
            E: 'e +::ormlite::Executor<'e, Database = #db>
        {
            Box::pin(async move {
                ::ormlite::query_as::<#db, Self>(#query)
                    .bind(id)
                    .fetch_one(db)
                    .await
                    .map_err(::ormlite::Error::from)
            })
        }
    }
}

pub fn impl_Model__builder(attr: &ModelMeta) -> TokenStream {
    let partial_model = &attr.builder_struct();
    quote! {
        fn builder() -> #partial_model<'static> {
            #partial_model::default()
        }
    }
}

pub fn impl_Model__update_partial(attr: &ModelMeta) -> TokenStream {
    let partial_model = &attr.builder_struct();
    quote! {
        /// Creates a builder for an *UPDATE* query that only updates changed fields
        fn update_partial(&self) -> #partial_model<'_> {
            let mut partial = #partial_model::default();
            partial.updating = Some(&self);
            partial
        }
    }
}


pub fn impl_Model__create_table(db: &dyn OrmliteCodegen, attr: &ModelMeta) -> TokenStream {
    let db = db.database_ts();
    let box_future = crate::util::box_fut_ts();
    let query = build_create_table_query(&attr.table);

    let documentation = format!(r#"Creates this table in the database, if it doesn't already exits.
This method is intended for drafting and tests, not anything you plan to release.
    
This is the query that will be executed:

```sql
{query}
```"#);

    quote!{
        #[doc = #documentation]
        fn create_table<'e, E>(conn: E) -> #box_future<'e, ::ormlite::Result<()>>
        where
            E: 'e + ::ormlite::Executor<'e, Database = #db>
        {
            use ::ormlite::Connection;
            use ::ormlite::model::*;
            Box::pin(async move {
                ::ormlite::query(&#query)
                    .execute(conn)
                    .await?;
                Ok(())
           })
        }
    
    }
}


fn build_create_table_query(table_meta: &TableMeta) -> String {
    use std::fmt::Write;

    let mut query = String::new();

    writeln!(query, "CREATE TABLE IF NOT EXISTS `{}` (", table_meta.name);

    let col_defs = table_meta.database_columns()
        .map(|c| format!("  {}", c.describe()))
        .join(",\n");

    write!(query, "{col_defs}\n);");

    query
}