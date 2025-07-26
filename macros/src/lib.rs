extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use shared::*;
use std::collections::BTreeMap;
use surrealdb_core::sql::parse;

mod lang;
mod shared;

#[doc(hidden)]
#[proc_macro]
#[rustfmt::skip]
pub fn sql(input: TokenStream) -> TokenStream {
    sql_impl(input).unwrap_or_else(|e| e)
}

#[rustfmt::skip]
fn sql_impl(input: TokenStream) -> Result<TokenStream, TokenStream> {
    let mut variables = BTreeMap::new();
    let sql = process_sql(input.clone(), Some(&mut variables))?;
    // #[cfg(not(debug_assertions))]
    if let Err(err) = parse(sql.as_str()) {
        println!("{}", &sql);
        let err = proc_macro2::Literal::string(err.to_string().as_str());
        let out = quote_spanned! {
             Span::call_site() =>
             compile_error!(#err #sql);
        };
        return Ok(out.into());
    }
    println!("{}", &sql);
    let query  = proc_macro2::Literal::string(sql.as_str());
    let vars = variables.into_iter().map(|(name, ident)| quote! {.put(#name, #ident)}).collect::<Vec<_>>();
    let out = quote! {
        (#query, surqx::Vars::new()#(#vars)*)
    };
    let out: TokenStream = out.into();
    Ok(out)
}
