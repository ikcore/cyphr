
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Lit, Expr, ExprLit};

fn get_rel_meta(ast: &DeriveInput) -> (String, Option<String>, Option<String>) {
    let mut rel_type: Option<String> = None;
    let mut from: Option<String> = None;
    let mut to: Option<String> = None;

    for attr in &ast.attrs {
        if attr.path().is_ident("cyphr") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        rel_type = Some(s.value());
                    }
                } else if meta.path.is_ident("from") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        from = Some(s.value());
                    }
                } else if meta.path.is_ident("to") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        to = Some(s.value());
                    }
                }
                Ok(())
            });
        }
    }

    let ty = rel_type.unwrap_or_else(|| ast.ident.to_string());
    (ty, from, to)
}

pub fn expand(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let (rel_type, from_label, to_label) = get_rel_meta(&ast);

    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(&ast, "CyphrRelation only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&ast, "CyphrRelation only supports structs")
                .to_compile_error()
                .into();
        }
    };

    let mut inits = Vec::new();

    for f in fields {
        let ident = f.ident.as_ref().unwrap();
        let key = ident.to_string();
        let ty = &f.ty;

        let mut prop_key = key.clone();
        for attr in &f.attrs {
            if attr.path().is_ident("cyphr") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("prop") {
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

        inits.push(quote! {
            #ident: {
                let v = cyphr_core::props::rel_prop(rel, #prop_key)
                    .ok_or_else(|| cyphr_core::error::CyphrError::missing_property(#prop_key, <Self as cyphr_core::traits::CyphrRelation>::TYPE))?;
                <#ty as cyphr_core::traits::FromCyphrValue>::from_value(v)
                    .map_err(|e| e.with_context(format!("{}::{} (prop '{}')", #rel_type, #key, #prop_key)))?
            }
        });
    }

    let from_label_tokens = match from_label {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };
    let to_label_tokens = match to_label {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    };

    let expanded = quote! {
        impl cyphr_core::traits::CyphrRelation for #name {
            const TYPE: &'static str = #rel_type;
            const FROM_LABEL: Option<&'static str> = #from_label_tokens;
            const TO_LABEL: Option<&'static str> = #to_label_tokens;

            fn from_rel(rel: &neo4rs::BoltRelation) -> Result<Self, cyphr_core::error::CyphrError> {
                Ok(Self {
                    #(#inits,)*
                })
            }
        }

        impl cyphr_core::traits::FromCyphrValue for #name {
            fn from_value(value: neo4rs::BoltType) -> Result<Self, cyphr_core::error::CyphrError> {
                match value {
                    neo4rs::BoltType::Relation(r) => <Self as cyphr_core::traits::CyphrRelation>::from_rel(&r),
                    other => Err(cyphr_core::error::CyphrError::type_mismatch(
                        "Relationship", cyphr_core::value::type_name(&other), #rel_type,
                    )),
                }
            }
        }
    };

    expanded.into()
}
