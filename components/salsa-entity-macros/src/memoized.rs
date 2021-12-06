use proc_macro2::Literal;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{ItemFn, ReturnType, Token};

use crate::configuration::{self, Configuration, CycleRecoveryStrategy};

// #[salsa::memoized(in Jar0)]
// fn my_func(db: &dyn Jar0Db, input1: u32, input2: u32) -> String {
//     format!("Hello, world")
// }

pub(crate) fn memoized(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let item_fn = syn::parse_macro_input!(input as ItemFn);
    let struct_item = configuration_struct(&args, &item_fn);
    let configuration = fn_configuration(&args, &item_fn);
    let struct_item_ident = &struct_item.ident;
    let struct_ty: syn::Type = parse_quote!(#struct_item_ident);
    let configuration_impl = configuration.to_impl(&struct_ty);
    let ingredients_for_impl = ingredients_for_impl(&args, &struct_ty);
    let (getter, setter) = wrapper_fns(&args, &item_fn, &struct_ty);

    proc_macro::TokenStream::from(quote! {
        #struct_item
        #configuration_impl
        #ingredients_for_impl
        #getter
        #setter
    })
}

struct Args {
    _in_token: Token![in],
    jar_ty: syn::Type,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            _in_token: Parse::parse(input)?,
            jar_ty: Parse::parse(input)?,
        })
    }
}

fn key_tuple_ty(item_fn: &syn::ItemFn) -> syn::Type {
    let arg_tys = item_fn.sig.inputs.iter().skip(1).map(|arg| match arg {
        syn::FnArg::Receiver(_) => unreachable!(),
        syn::FnArg::Typed(pat_ty) => pat_ty.ty.clone(),
    });

    parse_quote!(
        (#(#arg_tys,)*)
    )
}

fn configuration_struct(args: &Args, item_fn: &syn::ItemFn) -> syn::ItemStruct {
    let fn_name = item_fn.sig.ident.clone();
    let key_tuple_ty = key_tuple_ty(item_fn);
    parse_quote! {
        #[allow(non_camel_case_types)]
        pub struct #fn_name {
            intern_map: salsa::interned::InternedIngredient<salsa::Id, #key_tuple_ty>,
            function: salsa::function::FunctionIngredient<Self>,
        }
    }
}

fn fn_configuration(args: &Args, item_fn: &syn::ItemFn) -> Configuration {
    let jar_ty = args.jar_ty.clone();
    let key_ty = parse_quote!(salsa::id::Id);
    let value_ty = configuration::value_ty(&item_fn.sig);
    let ident_span = item_fn.sig.ident.span();

    // FIXME: these are hardcoded for now
    let cycle_strategy = CycleRecoveryStrategy::Panic;
    let memoize_value = true;

    let backdate_fn = configuration::should_backdate_value_fn(memoize_value);
    let recover_fn = configuration::panic_cycle_recovery_fn();

    // The type of the configuration struct; this has the same name as the fn itself.
    let fn_ty = item_fn.sig.ident.clone();

    // Make a copy of the fn with a different name; we will invoke this from `execute`.
    // We need to change the name because, otherwise, if the function invoked itself
    // recursively it would not go through the query system.
    let inner_fn_name = &syn::Ident::new("__fn", item_fn.sig.ident.span());
    let mut inner_fn = item_fn.clone();
    inner_fn.sig.ident = inner_fn_name.clone();

    // Create the `execute` function, which (a) maps from the interned id to the actual
    // keys and then (b) invokes the function itself (which we embed within).
    let indices = (0..item_fn.sig.inputs.len() - 1).map(|i| Literal::usize_unsuffixed(i));
    let execute_fn = parse_quote! {
        fn execute(__db: &salsa::function::DynDb<Self>, __id: Self::Key) -> Self::Value {
            #inner_fn

            let (__jar, __runtime) = salsa::storage::HasJar::jar(__db);
            let __ingredients =
                <_ as salsa::storage::HasIngredientsFor<#fn_ty>>::ingredient(__jar);
            let __key = __ingredients.intern_map.data(__runtime, __id).clone();
            #inner_fn_name(__db, #(__key.#indices),*)
        }
    };

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

fn ingredients_for_impl(args: &Args, struct_ty: &syn::Type) -> syn::ItemImpl {
    let jar_ty = &args.jar_ty;
    parse_quote! {
        impl salsa::storage::IngredientsFor for #struct_ty {
            type Ingredients = Self;
            type Jar = #jar_ty;

            fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
            where
                DB: salsa::DbWithJar<Self::Jar> + salsa::storage::JarFromJars<Self::Jar>,
            {
                Self {
                    intern_map: {
                        let index = ingredients.push(|jars| {
                            let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                            let ingredients =
                                <_ as salsa::storage::HasIngredientsFor<Self::Ingredients>>::ingredient(jar);
                            &ingredients.intern_map
                        });
                        salsa::interned::InternedIngredient::new(index)
                    },

                    function: {
                        let index = ingredients.push(|jars| {
                            let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                            let ingredients =
                                <_ as salsa::storage::HasIngredientsFor<Self::Ingredients>>::ingredient(jar);
                            &ingredients.function
                        });
                        salsa::function::FunctionIngredient::new(index)
                    },
                }
            }
        }
    }
}

fn wrapper_fns(
    args: &Args,
    item_fn: &syn::ItemFn,
    struct_ty: &syn::Type,
) -> (syn::ItemFn, syn::ItemImpl) {
    let value_arg = syn::Ident::new("__value", item_fn.sig.output.span());

    let (getter_block, setter_block) = wrapper_fn_bodies(args, item_fn, struct_ty, &value_arg)
        .unwrap_or_else(|msg| {
            let msg = proc_macro2::Literal::string(msg);
            (
                parse_quote_spanned! {
                    item_fn.sig.span() => {compile_error!(#msg)}
                },
                parse_quote!({}),
            )
        });

    // The "getter" has same signature as the original:
    let mut getter_fn = item_fn.clone();
    getter_fn.block = Box::new(getter_block);

    // The setter has *always* the same signature as the original:
    // but it takes a value arg and has no return type.
    let mut setter_sig = item_fn.sig.clone();
    let value_ty = configuration::value_ty(&item_fn.sig);
    setter_sig.ident = syn::Ident::new("set", item_fn.sig.ident.span());
    match &mut setter_sig.inputs[0] {
        // change from `&dyn ...` to `&mut dyn...`
        syn::FnArg::Receiver(_) => (),
        syn::FnArg::Typed(pat_ty) => match &mut *pat_ty.ty {
            syn::Type::Reference(ty) => {
                ty.mutability = Some(Token![mut](ty.and_token.span()));
            }
            _ => {}
        },
    }
    setter_sig.inputs.push(parse_quote!(#value_arg: #value_ty));
    setter_sig.output = ReturnType::Default;
    let mut setter_fn = syn::ImplItemMethod {
        attrs: vec![],
        vis: item_fn.vis.clone(),
        defaultness: None,
        sig: setter_sig,
        block: setter_block,
    };
    let setter_impl: syn::ItemImpl = parse_quote! {
        impl #struct_ty {
            #setter_fn
        }
    };

    (getter_fn, setter_impl)
}

fn wrapper_fn_bodies(
    args: &Args,
    item_fn: &syn::ItemFn,
    struct_ty: &syn::Type,
    value_arg: &syn::Ident,
) -> Result<(syn::Block, syn::Block), &'static str> {
    // Find the name `db` that user gave to the second argument.
    // They can't have done any "funny business" (such as a pattern
    // like `(db, _)` or whatever) or we get an error.
    let db_var = if item_fn.sig.inputs.len() == 0 {
        Err("method needs a database argument")
    } else {
        match &item_fn.sig.inputs[0] {
            syn::FnArg::Receiver(r) => Err("first argment be the database"),
            syn::FnArg::Typed(ty) => match &*ty.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err("first argment must be given a name"),
            },
        }
    };
    let db_var = db_var?;

    let arg_names: Result<Vec<syn::Ident>, &str> = item_fn
        .sig
        .inputs
        .iter()
        .skip(1)
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => Err("first argment be the database"),
            syn::FnArg::Typed(pat_ty) => match &*pat_ty.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err("all arguments must be given names"),
            },
        })
        .collect();
    let arg_names = arg_names?;

    let getter: syn::Block = parse_quote! {
        {
            let (__jar, __runtime) = salsa::storage::HasJar::jar(#db_var);
            let __ingredients = <_ as salsa::storage::HasIngredientsFor<#struct_ty>>::ingredient(__jar);
            let __key = __ingredients.intern_map.intern(__runtime, (#(#arg_names,)*));
            __ingredients.function.fetch(#db_var, __key)
        }
    };

    let setter: syn::Block = parse_quote! {
        {
            let (__jar, __runtime) = salsa::storage::HasJar::jar_mut(#db_var);
            let __ingredients = <_ as salsa::storage::HasIngredientsFor<#struct_ty>>::ingredient_mut(__jar);
            let __key = __ingredients.intern_map.intern(__runtime, (#(#arg_names,)*));
            __ingredients.function.store(__runtime, __key, #value_arg, salsa::Durability::LOW)
        }
    };

    Ok((getter, setter))
}
