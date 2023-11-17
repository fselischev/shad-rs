use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Scan, attributes(scan))]
pub fn derive_scan(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let default_impl = quote! {
        impl Scan for #ident {
            fn get_objects(&self) -> Vec<usize> {
                vec![]
            }
        }
    };

    let custom_impl = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .next()
                .map(|f| &f.ident)
                .map(|field_ident| {
                    quote! {
                        impl Scan for #ident {
                            fn get_objects(&self) -> Vec<usize> {
                                self.#field_ident.get_objects()
                            }
                        }
                    }
                })
        } else {
            None
        }
    } else {
        None
    };

    custom_impl.unwrap_or(default_impl).into()
}
