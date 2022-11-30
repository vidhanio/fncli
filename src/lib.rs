//! An attribute macro to simplify writing simple command line applications.
//!
//! # Examples
//!
//! ```no_run
//! #[fncli::cli]
//! fn main(a: i32, b: i32) {
//!     println!("{}", a + b);
//! }
//! ```
//!
//! ```bash session
//! $ cargo run 1 2
//! 3
//! ```

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]

use std::convert::identity;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::{
    parse::Parser, parse_macro_input::ParseMacroInput, spanned::Spanned, AttributeArgs, Error,
    FnArg, ItemFn, PatType, Signature,
};

/// The `cli` attribute macro.
#[proc_macro_attribute]
pub fn cli(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse(attr.into(), item.into())
        .map_or_else(|e| e.to_compile_error(), identity)
        .into()
}

fn parse(attr: TokenStream2, item: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let attr = AttributeArgs::parse.parse2(attr)?;
    let item = ItemFn::parse.parse2(item)?;

    if !attr.is_empty() {
        return Err(Error::new(attr[0].span(), "unexpected attribute argument"));
    }

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = &item;

    let Signature {
        constness,
        asyncness,
        unsafety,
        abi,
        fn_token,
        ident,
        generics,
        paren_token: _,
        inputs,
        variadic,
        output,
    } = &sig;

    if variadic.is_some() {
        return Err(Error::new(variadic.span(), "unexpected variadic argument"));
    }

    let pat_types = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(r) => Err(Error::new(r.span(), "unexpected `self` argument")),
            FnArg::Typed(p) => Ok(p),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let arg_patterns = pat_types.iter().map(
        |PatType {
             attrs,
             pat,
             colon_token: _,
             ty: _,
         }| quote_spanned!(pat.span()=> #(#attrs)* #pat),
    );

    let pattern_tuple = quote_spanned!(inputs.span()=> (#(#arg_patterns),*));
    let args = parse_args(&pat_types);
    let help_message = help(&pat_types);
    let len_error = format!("too many arguments (expected {})", pat_types.len());

    Ok(quote_spanned! {item.span()=>
        #(#attrs)*
        #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics() #output {
            #[allow(clippy::let_unit_value)]
            #[allow(unused_parens)]
            let #pattern_tuple = {
                use ::std::iter::Iterator;

                let mut args = ::std::env::args();

                let cmd = args.next().expect("should have a command name");

                let exit = |err: &str| -> ! {
                    eprintln!("{}", err);
                    eprintln!();
                    eprintln!(#help_message, cmd);
                    ::std::process::exit(1)
                };

                let tuple = (#(#args),*);

                if args.next().is_some() {
                    exit(#len_error);
                }

                tuple
            };

            #block
        }
    })
}

fn help(pat_types: &[&PatType]) -> String {
    "USAGE:\n    {}".to_owned()
        + &pat_types
            .iter()
            .map(
                |&PatType {
                     attrs: _,
                     pat,
                     colon_token: _,
                     ty,
                 }| {
                    format!(" <{}: {}>", pat.to_token_stream(), ty.to_token_stream())
                        .replace('{', "{{")
                        .replace('}', "}}")
                },
            )
            .collect::<String>()
}

fn parse_args<'a>(inputs: &'a [&PatType]) -> impl Iterator<Item = TokenStream2> + 'a {
    inputs.iter().map(
        |p @ PatType {
             attrs: _,
             pat,
             colon_token: _,
             ty,
         }| {
            quote_spanned! {p.span()=>
                <#ty as ::std::str::FromStr>::from_str(
                    &args.next().unwrap_or_else(
                        || exit(::std::concat!(
                            "missing argument: `",
                            stringify!(#pat),
                            ": ",
                            stringify!(#ty),
                            "`",
                        ))
                    )
                )
                .unwrap_or_else(
                    |e| exit(&format!(
                            "{} ({:?})",
                            ::std::concat!(
                                "failed to parse argument: `",
                                stringify!(#pat),
                                ": ",
                                stringify!(#ty),
                                "`",
                            ),
                            e,
                        ))
                )
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::*;

    #[test]
    fn basic() {
        let attr = TokenStream2::new();
        let item = quote! {
            #[argm::main]
            fn main(a: i32, b: i32) {
                a + b;
            }
        };

        assert!(parse(attr, item).is_ok());
    }
}
