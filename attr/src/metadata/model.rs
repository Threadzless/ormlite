use crate::metadata::column::ColumnMeta;
use crate::metadata::table::TableMeta;
use crate::Ident;
use crate::TableAttr;
use syn::DeriveInput;

/// Metadata used for IntoArguments, TableMeta, and (subset of) Model
#[derive(Debug, Clone)]
pub struct ModelMeta {
    pub table: TableMeta,
    pub insert_struct: Option<Ident>,
    pub pkey: ColumnMeta,
}

impl ModelMeta {
    pub fn builder_struct(&self) -> Ident {
        Ident::from(format!("{}Builder", self.ident))
    }

    pub fn database_columns_except_pkey(&self) -> impl Iterator<Item = &ColumnMeta> + '_ {
        self.columns
            .iter()
            .filter(|&c| !c.skip)
            .filter(|&c| self.pkey.name != c.name)
    }

    pub fn from_derive(ast: &DeriveInput) -> Self {
        let attrs = TableAttr::from_attrs(&ast.attrs);
        let table = TableMeta::new(ast, &attrs);
        let pkey = table.pkey.as_deref().unwrap_or_else(|| panic!(
            "No column marked with #[ormlite(primary_key)], and no column named id, uuid, {0}_id, or {0}_uuid",
            table.name,
        ));
        let mut insert_struct = None;
        for attr in attrs {
            if let Some(v) = attr.insert {
                insert_struct = Some(v.value());
            }
            if let Some(v) = attr.insertable {
                insert_struct = Some(v.to_string());
            }
        }
        let pkey = table.columns.iter().find(|&c| c.name == pkey).unwrap().clone();
        let insert_struct = insert_struct.map(Ident::from);
        Self {
            table,
            insert_struct,
            pkey,
        }
    }

    #[doc(hidden)]
    pub fn mock(name: &str, columns: Vec<ColumnMeta>) -> Self {
        let inner = TableMeta::mock(name, columns);
        Self {
            pkey: inner.columns.iter().find(|c| c.name == "id").unwrap().clone(),
            table: inner,
            insert_struct: None,
        }
    }
}

impl std::ops::Deref for ModelMeta {
    type Target = TableMeta;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::ItemStruct;

    #[test]
    fn test_decode_metadata() {
        let ast = syn::parse_str::<ItemStruct>(
            r#"struct User {
            #[ormlite(column = "Id")]
            id: i32,
        }"#,
        )
        .unwrap();
        let input = DeriveInput::from(ast);
        let meta = ModelMeta::from_derive(&input);
        assert_eq!(meta.pkey.name, "Id");
    }
}
