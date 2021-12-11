//! This crate provides salsa's macros and attributes.

#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

macro_rules! parse_quote {
    ($($inp:tt)*) => {
        syn::parse2(quote!{$($inp)*}).unwrap_or_else(|err| {
            panic!("failed to parse at {}:{}:{}: {}", file!(), line!(), column!(), err)
        })
    }
}

macro_rules! parse_quote_spanned {
    ($($inp:tt)*) => {
        syn::parse2(quote_spanned!{$($inp)*}).unwrap_or_else(|err| {
            panic!("failed to parse at {}:{}:{}: {}", file!(), line!(), column!(), err)
        })
    }
}

mod component;
mod configuration;
mod data_item;
mod db;
mod entity;
mod entity2;
mod interned;
mod jar;
mod memoized;
mod options;

#[proc_macro_attribute]
pub fn jar(args: TokenStream, input: TokenStream) -> TokenStream {
    jar::jar(args, input)
}

#[proc_macro_attribute]
pub fn db(args: TokenStream, input: TokenStream) -> TokenStream {
    db::db(args, input)
}

#[proc_macro_attribute]
pub fn entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entity::entity(args, input)
}

#[proc_macro]
pub fn entity2(input: TokenStream) -> TokenStream {
    entity2::entity(input)
}

#[proc_macro_attribute]
pub fn interned(args: TokenStream, input: TokenStream) -> TokenStream {
    interned::interned(args, input)
}

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component::component(args, input)
}

#[proc_macro_attribute]
pub fn memoized(args: TokenStream, input: TokenStream) -> TokenStream {
    memoized::memoized(args, input)
}
