//! An atribute macro to simplify writing simple command line applications.
//!
//! # Examples
//!
//! ```no_run
//! #[mainargs::mainargs]
//! fn main(a: i32, b: i32) {
//!     println!("{}", a + b);
//! }
//!
//! // $ cargo run 1 2
//! // 3
//! ```
//!
//! ```no_run
//! use std::str::FromStr;
//! struct Time {
//!     hour: u8,
//!     minute: u8,
//! }
//!
//! impl FromStr for Time {
//!     type Err = &'static str;
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         let (hour, minute) = s.split_once(':').ok_or("should have a colon")?;
//!         let hour = hour.parse().map_err(|_| "invalid hour")?;
//!         let minute = minute.parse().map_err(|_| "invalid minute")?;
//!         Ok(Time { hour, minute })
//!     }
//! }
//!
//! #[mainargs::mainargs]
//! fn main(time: Time) {
//!     println!("{} hours, {} minutes", time.hour, time.minute);
//! }
//!
//! // $ cargo run 12:34
//! // 12 hours, 34 minutes
//!
//! // $ cargo run 12
//! // failed to parse argument `time`: "should have a colon"

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::Parser, parse_macro_input::ParseMacroInput, spanned::Spanned, AttributeArgs, Error,
    ItemFn, PatType, Signature,
};

/// The main function attribute macro.
#[proc_macro_attribute]
pub fn mainargs(attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse(attr.into(), item.into()) {
        Ok(stream) => stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
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
    } = item;

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
    } = sig;

    if variadic.is_some() {
        return Err(Error::new(variadic.span(), "unexpected variadic function"));
    }

    let arg_patterns = inputs.iter().map(|arg| match arg {
        syn::FnArg::Receiver(_) => {
            Error::new(arg.span(), "unexpected `self` argument").to_compile_error()
        }
        syn::FnArg::Typed(PatType { pat, .. }) => quote! { #pat },
    });

    let tuple = quote! {
        (#(#arg_patterns),*)
    };

    let args = inputs.iter().map(|arg| {
            match arg {
                syn::FnArg::Receiver(r) => {
                    Error::new(r.span(), "unexpected `self` argument").to_compile_error()
                }
                syn::FnArg::Typed(pt) => {
                    let pat = &pt.pat;
                    let ty = &pt.ty;
                    quote! {
                        {
                            let arg = args.next().expect(::std::concat!("missing argument", " ", "`", stringify!(#pat), "`"));
                            <#ty as ::std::str::FromStr>::from_str(&arg).expect(::std::concat!("failed to parse argument", " ", "`", stringify!(#pat), "`"))
                        }
                    }
                }
            }
        });

    Ok(quote! {
        #(#attrs)*
        #vis #constness #asyncness #unsafety #abi #fn_token #ident #generics() #output {
            let #tuple = {
                let mut args = ::std::env::args().skip(1);
                (#(#args),*)
            };

            #block
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
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
