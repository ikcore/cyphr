
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Expr, ExprLit, Lit};

struct FieldInfo {
    ident: syn::Ident,
    prop_key: String,
    skip: bool,
}

fn parse_field(f: &syn::Field) -> FieldInfo {
    let ident = f.ident.as_ref().unwrap().clone();
    let mut prop_key = ident.to_string();
    let mut skip = false;

    for attr in &f.attrs {
        if attr.path().is_ident("cyphr") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") || meta.path.is_ident("id") {
                    skip = true;
                } else if meta.path.is_ident("prop") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        prop_key = s.value();
                    }
                }
                Ok(())
            });
        }
    }

    FieldInfo { ident, prop_key, skip }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(&ast, "ToCyphrParams only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&ast, "ToCyphrParams only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut inserts = Vec::new();

    for f in fields {
        let info = parse_field(f);
        if info.skip {
            continue;
        }
        let ident = &info.ident;
        let key = &info.prop_key;
        inserts.push(quote! {
            map.insert(#key.to_string(), cyphr_core::traits::IntoCyphrValue::into_value(self.#ident));
        });
    }

    let expanded = quote! {
        impl cyphr_core::traits::ToCyphrParams for #name {
            fn to_params(self) -> std::collections::HashMap<String, neo4rs::BoltType> {
                let mut map = std::collections::HashMap::new();
                #(#inserts)*
                map
            }
        }
    };

    expanded.into()
}
