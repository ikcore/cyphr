
use proc_macro::TokenStream;
use quote::quote;

/// Convert a token stream like:
///   MATCH (u:User) RETURN u
/// into a &'static str with normalized whitespace.
///
/// This is intentionally simple to keep Cypher flexible.
pub fn expand(input: TokenStream) -> TokenStream {
    let s = input.to_string();

    // TokenStream::to_string() inserts spaces around punctuation; normalize a bit.
    // Keep it simple and predictable.
    let mut out = s.replace(" ,", ",")
        .replace(" :", ":")
        .replace(": ", ":")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace("  ", " ");

    // Preserve newlines: users may include them via explicit \n in tokens; otherwise to_string flattens.
    // We'll just trim.
    out = out.trim().to_string();

    quote! { #out }.into()
}
