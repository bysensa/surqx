extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use shared::*;
use std::collections::BTreeMap;

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
    let sql = sql_from_macro(input.clone(), Some(&mut variables))?;
    // #[cfg(not(debug_assertions))]
    // if let Err(err) = parse(sql.replace("\n", "").trim()) { 
    //     panic!("{}", &err.to_string());
    //     let err = proc_macro2::Literal::string(err.to_string().as_str());
    //     let out = quote_spanned! {
    //          Span::call_site() =>
    //          compile_error!(#err);
    //     };
    //     return Ok(out.into());
    // }
    let query  = proc_macro2::Literal::string(sql.as_str());
    let vars = variables.into_iter().map(|(name, ident)| quote! {.put(#name, #ident)}).collect::<Vec<_>>();
    let out = quote! {
        (#query, surreal_query::Vars::new()#(#vars)*)
    };
    let out: TokenStream = out.into();
    Ok(out)
}
