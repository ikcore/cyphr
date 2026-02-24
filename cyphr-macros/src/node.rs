
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Expr, ExprLit, Lit};

fn get_label(ast: &DeriveInput) -> String {
    for attr in &ast.attrs {
        if attr.path().is_ident("cyphr") {
            let mut label = None;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("label") {
                    let value = meta.value()?;
                    let expr: Expr = value.parse()?;
                    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
                        label = Some(s.value());
                    }
                }
                Ok(())
            });
            if let Some(l) = label {
                return l;
            }
        }
    }
    ast.ident.to_string()
}

pub fn expand(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let label = get_label(&ast);

    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(&ast, "CyphrNode only supports structs with named fields")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&ast, "CyphrNode only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // For each field, generate: field: <ty as FromCyphrValue>::from_value(node_prop(...).ok_or(..)?)?
    let mut inits = Vec::new();

    for f in fields {
        let ident = f.ident.as_ref().unwrap();
        let key = ident.to_string();
        let ty = &f.ty;

        // Support #[cyphr(id)] or #[cyphr(prop="...")]
        let mut prop_key = key.clone();
        for attr in &f.attrs {
            if attr.path().is_ident("cyphr") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("id") {
                        // keep same key; marker only for now
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

        inits.push(quote! {
            #ident: {
                let v = cyphr_core::props::node_prop(node, #prop_key)
                    .ok_or_else(|| cyphr_core::error::CyphrError::missing_property(#prop_key, <Self as cyphr_core::traits::CyphrNode>::LABEL))?;
                <#ty as cyphr_core::traits::FromCyphrValue>::from_value(v)
                    .map_err(|e| e.with_context(format!("{}::{} (prop '{}')", #label, #key, #prop_key)))?
            }
        });
    }

    let expanded = quote! {
        impl cyphr_core::traits::CyphrNode for #name {
            const LABEL: &'static str = #label;

            fn from_node(node: &neo4rs::BoltNode) -> Result<Self, cyphr_core::error::CyphrError> {
                Ok(Self {
                    #(#inits,)*
                })
            }
        }

        impl cyphr_core::traits::FromCyphrValue for #name {
            fn from_value(value: neo4rs::BoltType) -> Result<Self, cyphr_core::error::CyphrError> {
                match value {
                    neo4rs::BoltType::Node(n) => <Self as cyphr_core::traits::CyphrNode>::from_node(&n),
                    other => Err(cyphr_core::error::CyphrError::type_mismatch(
                        "Node", cyphr_core::value::type_name(&other), #label,
                    )),
                }
            }
        }
    };

    expanded.into()
}
