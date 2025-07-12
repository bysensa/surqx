use nanoid::nanoid;
use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::{
    collections::BTreeMap,
    fmt::{Display, Write},
};

pub const ID_ALPHABET: [char; 63] = [
    '_', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A',
    'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
    'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub(crate) fn punct(p: char) -> TokenTree {
    TokenTree::Punct(Punct::new(p, Spacing::Joint))
}

pub(crate) fn ident(s: &str) -> TokenTree {
    TokenTree::Ident(Ident::new(s, Span::call_site()))
}

pub(crate) fn braces(t: impl IntoIterator<Item = TokenTree>) -> TokenTree {
    TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::from_iter(t)))
}

pub(crate) fn string(s: &str) -> TokenTree {
    TokenTree::Literal(Literal::string(s))
}

/// Turn the tokens into a string with reconstructed whitespace.
///
/// If `variables` is set, variables (syntax: 'var) are replaced by `_RUST_var` and inserted in the map.
pub(crate) fn sql_from_macro(
    input: TokenStream,
    variables: Option<&mut BTreeMap<String, proc_macro2::Ident>>,
) -> Result<String, TokenStream> {
    struct Location {
        first_indent: Option<usize>,
        line: usize,
        column: usize,
    }

    fn add_whitespace(
        query: &mut String,
        loc: &mut Location,
        span: Span,
    ) -> Result<(), TokenStream> {
        let line = span.line();
        let column = span.column();
        if line > loc.line {
            while line > loc.line {
                query.push('\n');
                loc.line += 1;
            }
            let first_indent = *loc.first_indent.get_or_insert(column);
            let indent = column.checked_sub(first_indent);
            let indent =
                indent.ok_or_else(|| compile_error(Some((span, span)), "invalid indent"))?;
            for _ in 0..indent {
                query.push(' ');
            }
            loc.column = column;
        } else if line == loc.line {
            while column > loc.column {
                query.push(' ');
                loc.column += 1;
            }
        }
        Ok(())
    }

    fn add_tokens(
        query: &mut String,
        loc: &mut Location,
        input: TokenStream,
        mut variables: Option<&mut BTreeMap<String, proc_macro2::Ident>>,
    ) -> Result<(), TokenStream> {
        let mut tokens = input.into_iter();
        let mut should_correct_location = true;
        while let Some(token) = tokens.next() {
            let span = token.span();
            if should_correct_location {
                loc.line = span.line();
                loc.column = span.column();
                should_correct_location = false;
            }
            add_whitespace(query, loc, span)?;
            match &token {
                TokenTree::Group(x) => {
                    let (start, end) = match x.delimiter() {
                        Delimiter::Parenthesis => ("(", ")"),
                        Delimiter::Brace => ("{", "}"),
                        Delimiter::Bracket => ("[", "]"),
                        Delimiter::None => ("", ""),
                    };
                    add_whitespace(query, loc, x.span_open())?;
                    query.push_str(start);
                    loc.column += start.len();
                    add_tokens(query, loc, x.stream(), variables.as_deref_mut())?;
                    add_whitespace(query, loc, x.span_close())?;
                    query.push_str(end);
                    loc.column += end.len();
                }
                TokenTree::Punct(x) => {
                    if variables.is_some() && x.as_char() == '&' && x.spacing() == Spacing::Alone {
                        let Some(TokenTree::Ident(ident)) = tokens.next() else {
                            unreachable!()
                        };
                        let id = nanoid!(8, &ID_ALPHABET);
                        let name = format!("{}_{}", ident.to_string(), id);
                        write!(query, "${name}").unwrap();
                        loc.column += name.chars().count() + 1;
                        if let Some(variables) = &mut variables {
                            let ident = proc_macro2::Ident::new(
                                ident.to_string().as_str(),
                                proc_macro2::Span::from(ident.span()),
                            );
                            variables.entry(name).or_insert(ident);
                        }
                    } else if x.as_char() == '#' && x.spacing() == Spacing::Joint {
                        // Convert '##' to '//', because otherwise it's
                        // impossible to use the Python operators '//' and '//='.
                        match tokens.next() {
                            Some(TokenTree::Punct(ref p)) if p.as_char() == '#' => {
                                query.push_str("//");
                                loc.column += 2;
                            }
                            Some(TokenTree::Punct(p)) => {
                                query.push(x.as_char());
                                query.push(p.as_char());
                                loc.column += 2;
                            }
                            _ => {
                                unreachable!();
                            }
                        }
                    } else {
                        query.push(x.as_char());
                        loc.column += 1;
                    }
                }
                TokenTree::Ident(x) => {
                    write!(query, "{x}").unwrap();
                    let end_span = token.span().end();
                    loc.line = end_span.line();
                    loc.column = end_span.column();
                }
                TokenTree::Literal(x) => {
                    let s = x.to_string();
                    // Remove space in prefixed strings like `f ".."`.
                    // (`f".."` is not allowed in some versions+editions of Rust.)
                    if s.starts_with('"')
                        && query.ends_with(' ')
                        && query[..query.len() - 1].ends_with(|c: char| c.is_ascii_alphabetic())
                    {
                        query.pop();
                    }
                    query.push_str(&s);
                    let end_span = token.span().end();
                    loc.line = end_span.line();
                    loc.column = end_span.column();
                }
            }
        }
        Ok(())
    }

    let mut sql = String::new();
    let mut location = Location {
        line: 0,
        column: 0,
        first_indent: None,
    };
    add_tokens(&mut sql, &mut location, input, variables)?;
    Ok(sql)
}

/// Create a compile_error!{} using two spans that mark the start and end of the error.
#[rustfmt::skip]
pub(crate) fn compile_error(spans: Option<(Span, Span)>, error: &(impl Display + ?Sized)) -> TokenStream  {
    let mut tokens = [
        punct(':'), punct(':'), ident("core"),
        punct(':'), punct(':'), ident("compile_error"),
        punct('!'), braces([string(&format!("sql: {error}"))]),
    ];
    if let Some((span1, span2)) = spans {
        for (i, t) in tokens.iter_mut().enumerate() {
            t.set_span(if i < 6 { span1 } else { span2 });
        }
    }
    TokenStream::from_iter(tokens)
}
