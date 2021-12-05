//! This crate provides salsa's macros and attributes.

#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

mod component;
mod configuration;
mod entity;
mod interned;
mod jar;

#[proc_macro_attribute]
pub fn jar(args: TokenStream, input: TokenStream) -> TokenStream {
    jar::jar(args, input)
}

#[proc_macro_attribute]
pub fn entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entity::entity(args, input)
}

#[proc_macro_attribute]
pub fn interned(args: TokenStream, input: TokenStream) -> TokenStream {
    interned::interned(args, input)
}

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component::component(args, input)
}
