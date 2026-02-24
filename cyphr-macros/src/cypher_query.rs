//! Implementation of the `cypher_query!` proc macro.
//!
//! Scans a Cypher token stream for `$ident` patterns, normalizes whitespace
//! (same rules as `cypher!`), and generates a `CyphrQuery::new("...").param(...)...`
//! expression with all discovered parameters auto-bound.

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use std::collections::HashSet;

/// Recursively walk a token stream (including groups like `(...)`, `{...}`)
/// and collect every `$ident` parameter into `params` (deduplicated by name).
fn scan_params(
    stream: proc_macro2::TokenStream,
    params: &mut Vec<proc_macro2::Ident>,
    seen: &mut HashSet<String>,
) {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == '$' => {
                if i + 1 < tokens.len() {
                    if let TokenTree::Ident(ident) = &tokens[i + 1] {
                        let name = ident.to_string();
                        if seen.insert(name) {
                            params.push(ident.clone());
                        }
                        i += 2;
                        continue;
                    }
                }
                i += 1;
            }
            TokenTree::Group(g) => {
                scan_params(g.stream(), params, seen);
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let s = input2.to_string();

    // Same whitespace normalization as cypher!, plus `$ ` → `$` for params.
    let out = s
        .replace(" ,", ",")
        .replace(" :", ":")
        .replace(": ", ":")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace("$ ", "$")
        .replace("  ", " ");
    let out = out.trim().to_string();

    // Scan tokens for $ident patterns
    let mut params: Vec<proc_macro2::Ident> = Vec::new();
    let mut seen = HashSet::new();
    scan_params(input2, &mut params, &mut seen);

    let param_calls = params.iter().map(|ident| {
        let name = ident.to_string();
        quote! { .param(#name, #ident) }
    });

    quote! {
        cyphr::query::CyphrQuery::new(#out) #(#param_calls)*
    }
    .into()
}
