use heck::{CamelCase, SnakeCase};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, ItemImpl, ItemStruct, Token};

use crate::configuration::{self, Configuration, CycleRecoveryStrategy};

// #[salsa::component(EntityComponent0 in Jar0)]
// impl Entity0 {
//     fn method(self, db: &dyn Jar0Db) -> String {
//         format!("Hello, world")
//     }
// }

pub(crate) fn component(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let item_impl = syn::parse_macro_input!(input as ItemImpl);
    component_contents(&args, &item_impl).into()
}

pub struct Args {
    component_ident: Ident,
    _in_token: Token![in],
    jar_ty: syn::Type,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            component_ident: Parse::parse(input)?,
            _in_token: Parse::parse(input)?,
            jar_ty: Parse::parse(input)?,
        })
    }
}

fn component_contents(args: &Args, item_impl: &ItemImpl) -> proc_macro2::TokenStream {
    let component_struct: ItemStruct =
        syn::parse2(component_struct(args)).expect("id_struct parse failed");

    quote! {
        #component_struct

    }
}

fn impl_items_and_method_structs(
    args: &Args,
    item_impl: &ItemImpl,
) -> (Vec<syn::ImplItem>, Vec<syn::Item>, Vec<syn::Field>) {
    let mut impl_items = vec![];
    let mut items: Vec<syn::Item> = vec![];
    let mut fields = vec![];

    for impl_item in &item_impl.items {
        match impl_item {
            syn::ImplItem::Const(_) | syn::ImplItem::Type(_) => impl_items.push(impl_item.clone()),

            syn::ImplItem::Method(method) => {
                let method_name = &method.sig.ident;

                // Generate a struct to "house" the configuration for this method.
                let struct_name = syn::Ident::new(
                    &format!("{}{}", args.component_ident, method_name).to_camel_case(),
                    method_name.span(),
                );
                items.push(
                    syn::parse2(quote! {
                        struct #struct_name;
                    })
                    .unwrap(),
                );

                // Generate the `impl Configuration for struct" item
                let struct_ty = syn::parse2(quote! { #struct_name }).unwrap();
                let configuration = method_configuration(args, item_impl, method);
                items.push(configuration.to_impl(&struct_ty).into());

                impl_items.push(syn::parse2(quote! {}).unwrap());
            }

            _ => {
                impl_items.push(syn::parse2(quote_spanned! {
                    impl_item.span() => compile_error!("only constants, methods, and types are permitted")
                }).unwrap());
            }
        }
    }

    (impl_items, items, fields)
}

fn component_struct(args: &Args) -> proc_macro2::TokenStream {
    let component_ident = &args.component_ident;
    quote! {
        pub struct #component_ident {
            method: salsa::function::FunctionIngredient<EntityComponent0_method>,
        }
    }
}

fn value_ty(method: &syn::ImplItemMethod) -> syn::Type {
    match &method.sig.output {
        syn::ReturnType::Default => syn::parse2(quote! { () }).unwrap(),
        syn::ReturnType::Type(_, ty) => syn::Type::clone(ty),
    }
}

fn method_configuration(
    args: &Args,
    item_impl: &syn::ItemImpl,
    method: &syn::ImplItemMethod,
) -> Configuration {
    let jar_ty = args.jar_ty.clone();
    let key_ty = syn::Type::clone(&item_impl.self_ty);
    let value_ty = value_ty(method);
    let ident_span = method.sig.ident.span();

    // FIXME: these are hardcoded for now
    let cycle_strategy = CycleRecoveryStrategy::Panic;
    let memoize_value = true;

    let trait_item = syn::TraitItemMethod {
        attrs: vec![],
        sig: method.sig.clone(),
        default: None,
        semi_token: Some(Token![;](ident_span)),
    };

    // Create the `execute` function. We have to do a bit of "funkiness" here because
    // we want the `self` to be the entity.
    let secret_trait_name = syn::Ident::new(
        &format!("__{}__", method.sig.ident.to_string().to_camel_case()),
        ident_span,
    );
    let execute_fn: syn::ImplItemMethod = syn::parse2(quote! {
        fn execute(db: &salsa::function::DynDb<Self>, key: Self::Key) -> Self::Value {
            trait #secret_trait_name {
                #trait_item
            }

            impl #secret_trait_name for #key_ty {
                #method
            }

            <Self::Key as #secret_trait_name>::method(key, db)
        }
    })
    .unwrap();

    let backdate_fn = configuration::should_backdate_value_fn(memoize_value);
    let recover_fn = configuration::panic_cycle_recovery_fn();

    Configuration {
        jar_ty,
        key_ty,
        value_ty,
        cycle_strategy,
        memoize_value,
        backdate_fn,
        execute_fn,
        recover_fn,
    }
}

fn method_wrappers(
    args: &Args,
    item_impl: &syn::ItemImpl,
    method: &syn::ImplItemMethod,
) -> (syn::ImplItemMethod, syn::ImplItemMethod) {
    // We need to generate something like this:
    //
    //     fn method(self, db: &dyn Jar0Db) -> String {
    //         let (jar, _) = salsa::storage::HasJar::jar(db);
    //         let component: &EntityComponent0 =
    //             <Jar0 as salsa::storage::HasIngredientsFor<EntityComponent0>>::ingredient(jar);
    //         component.method.fetch(db, self)
    //     }
    //
    // from the user's input:
    //
    //     fn method(self, db: &dyn Jar0Db) -> String {
    //         ...
    //     }
    //
    // As much as possible we try to copy the user's tokens and just let rustc handle
    // type checking and validation: but we *do* need to check that the method has at
    // least two arguments and get the name of the `db` parameter.

    // Find the name `db` that user gave to the second argument.
    // They can't have done any "funny business" (such as a pattern
    // like `(db, _)` or whatever) or we get an error.
    let db_var = if method.sig.inputs.len() != 2 {
        Err("method needs to have 2 arguments")
    } else {
        match &method.sig.inputs[1] {
            syn::FnArg::Receiver(r) => Err("second argment must not be self"),
            syn::FnArg::Typed(ty) => match &*ty.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err("second argment must be given a name"),
            },
        }
    };

    let block_tokens = match &db_var {
        Err(msg) => quote_spanned! { method.sig.span() =>
            compile_error!(#msg)
        },
        Ok(db_var) => quote! {
            let (jar, _) = salsa::storage::HasJar::jar(#db_var);
            let component: &EntityComponent0 =
                <Jar0 as salsa::storage::HasIngredientsFor<EntityComponent0>>::ingredient(jar);
            component.method.fetch(#db_var, self)
        },
    };

    // Now generate a `set_foo()` method.
    let mut set_sig = method.sig.clone();
    set_sig.ident = syn::Ident::new(
        &format!("set_{}", method.sig.ident),
        method.sig.ident.span(),
    );
    let value_ty = value_ty(method);
    set_sig
        .inputs
        .push(syn::parse2(quote! {value: #value_ty}).unwrap());
    set_sig.output = syn::ReturnType::Default;
    let set_block_tokens = match &db_var {
        Err(_) => quote! { () },
        Ok(db_var) => quote! {
            let (jar, _) = salsa::storage::HasJar::jar(#db_var);
            let component: &EntityComponent0 =
                <Jar0 as salsa::storage::HasIngredientsFor<EntityComponent0>>::ingredient(jar);
            component.method.set(#db_var, self, value)
        },
    };

    (
        syn::ImplItemMethod {
            attrs: method.attrs.clone(),
            vis: method.vis.clone(),
            defaultness: method.defaultness.clone(),
            sig: method.sig.clone(),
            block: syn::parse2(block_tokens).unwrap(),
        },
        syn::ImplItemMethod {
            attrs: method.attrs.clone(),
            vis: method.vis.clone(),
            defaultness: method.defaultness.clone(),
            sig: set_sig,
            block: syn::parse2(set_block_tokens).unwrap(),
        },
    )
}
