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
use quote::quote_spanned;
use syn::{
    parse::Parser, parse_macro_input::ParseMacroInput, spanned::Spanned, AttributeArgs, Error,
    ItemFn, PatType, Signature,
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
        return Err(Error::new(variadic.span(), "unexpected variadic function"));
    }

    let arg_patterns = inputs.iter().map(|arg| match arg {
        syn::FnArg::Receiver(_) => {
            Error::new(arg.span(), "unexpected `self` argument").to_compile_error()
        }
        syn::FnArg::Typed(PatType { pat, .. }) => quote_spanned!(pat.span()=> #pat),
    });

    let tuple = quote_spanned!(inputs.span()=> (#(#arg_patterns),*));

    let args = inputs.iter().map(|arg| match arg {
        syn::FnArg::Receiver(r) => {
            Error::new(r.span(), "unexpected `self` argument").to_compile_error()
        }
        syn::FnArg::Typed(pt) => {
            let pat = &pt.pat;
            let ty = &pt.ty;
            quote_spanned! {arg.span()=>
                {
                    let arg = args.next().expect(
                        ::std::concat!(
                            "missing argument",
                            " ",
                            "`",
                            stringify!(#pat),
                            ":",
                            " ",
                            stringify!(#ty),
                            "`"
                        )
                    );

                    <#ty as ::std::str::FromStr>::from_str(&arg).expect(
                        ::std::concat!(
                            "failed to parse argument",
                            " ",
                            "`",
                            stringify!(#pat),
                            ":",
                            " ",
                            stringify!(#ty),
                            "`",
                        )
                    )
                }
            }
        }
    });

    let len = inputs.len();

    Ok(quote_spanned! {item.span() =>
        #(#attrs)*
        #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics() #output {
            #[allow(clippy::let_unit_value)]
            #[allow(unused_parens)]
            let #tuple = {
                use ::std::iter::Iterator;

                let mut args = ::std::env::args().skip(1);

                let tuple = (#(#args),*);

                if args.next().is_some() {
                    ::std::panic!(::std::concat!(
                        "too many arguments",
                        " ",
                        "(expected",
                        " ",
                        #len,
                        " arguments)",
                    ));
                }

                tuple
            };

            #block
        }
    })
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
