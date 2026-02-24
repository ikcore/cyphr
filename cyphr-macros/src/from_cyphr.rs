
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

fn has_flatten(f: &syn::Field) -> bool {
    for attr in &f.attrs {
        if attr.path().is_ident("cyphr") {
            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("flatten") {
                    found = true;
                }
                Ok(())
            });
            if found {
                return true;
            }
        }
    }
    false
}

pub fn expand(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(&ast, "FromCyphr only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&ast, "FromCyphr only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let struct_name = name.to_string();
    let mut inits = Vec::new();

    for f in fields {
        let ident = f.ident.as_ref().unwrap();
        let key = ident.to_string();
        let ty = &f.ty;

        if has_flatten(f) {
            inits.push(quote! {
                #ident: <#ty as cyphr_core::traits::FromCyphr>::from_record(record)?
            });
            continue;
        }

        // If field type is Option<...>, allow missing key => None.
        // We do a *syntactic* check to keep it lightweight.
        let is_option = match ty {
            syn::Type::Path(p) => p.path.segments.first().map(|s| s.ident == "Option").unwrap_or(false),
            _ => false,
        };

        if is_option {
            inits.push(quote! {
                #ident: {
                    match cyphr_core::record::get_value(record, #key) {
                        None => None,
                        Some(v) => <#ty as cyphr_core::traits::FromCyphrValue>::from_value(v)
                            .map_err(|e| e.with_context(format!("{}::{}", #struct_name, #key)))?,
                    }
                }
            });
        } else {
            inits.push(quote! {
                #ident: {
                    let v = cyphr_core::record::get_value(record, #key)
                        .ok_or_else(|| cyphr_core::error::CyphrError::missing_field(#key, #struct_name))?;
                    <#ty as cyphr_core::traits::FromCyphrValue>::from_value(v)
                        .map_err(|e| e.with_context(format!("{}::{}", #struct_name, #key)))?
                }
            });
        }
    }

    let expanded = quote! {
        impl cyphr_core::traits::FromCyphr for #name {
            fn from_record(record: &neo4rs::Row) -> Result<Self, cyphr_core::error::CyphrError> {
                Ok(Self {
                    #(#inits,)*
                })
            }
        }
    };

    expanded.into()
}
