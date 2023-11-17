use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, FieldsNamed, Ident, LitStr, Type};

const TABLE_NAME: &str = "table_name";
const COLUMN_NAME: &str = "column_name";

#[proc_macro_derive(Object, attributes(table_name, column_name))]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input);

    let table_name = try_find_attr_value(TABLE_NAME, &attrs).unwrap_or_else(|| ident.to_string());

    let (field, col, ty) = parse_data(data);

    quote!(
        impl ::orm::Object for #ident {
            fn schema() -> &'static ::orm::object::Schema {
                &::orm::object::Schema {
                    type_name: stringify!(#ident),
                    table_name: #table_name,
                    attrs: &[
                        #(
                            ::orm::object::Attribute {
                                name: stringify!(#field),
                                col_name: #col,
                                data_type: <#ty as ::orm::data::AsDataType>::DATA_TYPE,
                            },
                        )*
                    ],
                }
            }

            fn as_table_row(&self) -> ::orm::storage::Row {
                vec![#((&self.#field).into()),*]
            }

            fn from_table_row(row: ::orm::storage::Row) -> Self {
                let mut row = row.into_iter();
                Self {
                    #(#field: ::orm::data::IntoDataType::into(row.next().unwrap())),*
                }
            }
        }
    )
    .into()
}

fn try_find_attr_value(ident: &str, attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if let Some(value) = {
            if attr.path.is_ident(ident) {
                if let Ok(lit) = attr.parse_args::<LitStr>() {
                    return Some(lit.value());
                }
            }

            None
        } {
            return Some(value);
        }
    }

    None
}

fn parse_data(data: Data) -> (Vec<Ident>, Vec<String>, Vec<Type>) {
    let mut idents = vec![];
    let mut cols = vec![];
    let mut types = vec![];

    match (match data {
        Data::Struct(s) => s,
        _ => panic!("Only structs are available to derive trait Object"),
    })
    .fields
    {
        syn::Fields::Named(FieldsNamed { named, .. }) => named.into_iter().for_each(|f| {
            cols.push(
                try_find_attr_value(COLUMN_NAME, &f.attrs)
                    .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string()),
            );
            idents.push(f.ident.unwrap());
            types.push(f.ty);
        }),
        syn::Fields::Unit => return (vec![], vec![], vec![]),
        syn::Fields::Unnamed(_) => panic!(),
    };

    (idents, cols, types)
}
