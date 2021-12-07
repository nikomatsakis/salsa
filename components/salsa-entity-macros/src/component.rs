use heck::CamelCase;
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

struct Args {
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
    let (impl_items, items, fields) = impl_items_and_method_structs(args, item_impl);
    let component_struct: ItemStruct = component_struct(args, &fields);
    let ingredients = ingredients_for_component_struct(args, &fields);

    let mut new_impl = item_impl.clone();
    new_impl.items = impl_items;

    quote! {
        #component_struct
        #ingredients
        #new_impl
        #(#items)*
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
                items.push(parse_quote! {
                    struct #struct_name;
                });

                // Generate the `impl Configuration for struct" item
                let struct_ty = parse_quote! { #struct_name };
                let configuration = method_configuration(args, item_impl, method);
                items.push(configuration.to_impl(&struct_ty).into());

                // Generate the getter/setter methods
                let (getter, setter) = method_wrappers(args, method);
                impl_items.push(getter.into());
                impl_items.push(setter.into());

                // Generate the field storing the ingredient
                fields.push(syn::Field {
                    attrs: vec![],
                    vis: syn::Visibility::Inherited,
                    ident: Some(method_name.clone()),
                    colon_token: Some(Token![:](method_name.span())),
                    ty: parse_quote!(salsa::function::FunctionIngredient<#struct_name>),
                });
            }

            _ => {
                impl_items.push(parse_quote_spanned! {
                    impl_item.span() => compile_error!("only constants, methods, and types are permitted")
                });
            }
        }
    }

    (impl_items, items, fields)
}

fn component_struct(args: &Args, fields: &[syn::Field]) -> syn::ItemStruct {
    let component_ident = &args.component_ident;
    parse_quote! {
        pub struct #component_ident {
            #(#fields),*
        }
    }
}

fn ingredients_for_component_struct(args: &Args, fields: &[syn::Field]) -> syn::ItemImpl {
    let component_ident = &args.component_ident;
    let jar_ty = &args.jar_ty;
    let field_names = fields.iter().map(|f| &f.ident);

    parse_quote! {
        impl salsa::storage::IngredientsFor for #component_ident {
            type Jar = #jar_ty;
            type Ingredients = Self;

            fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
            where
                DB: salsa::DbWithJar<Self::Jar> + salsa::storage::JarFromJars<Self::Jar>,
            {
                Self {
                    #(
                        #field_names: {
                            let index = ingredients.push(|jars| {
                                let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                                let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar);
                                &ingredients.#field_names
                            });
                            salsa::function::FunctionIngredient::new(index)
                        },
                    )*
                }
            }
        }
    }
}

fn method_configuration(
    args: &Args,
    item_impl: &syn::ItemImpl,
    method: &syn::ImplItemMethod,
) -> Configuration {
    let jar_ty = args.jar_ty.clone();
    let key_ty = syn::Type::clone(&item_impl.self_ty);
    let value_ty = configuration::value_ty(&method.sig);
    let method_name = &method.sig.ident;
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

    let mut impl_item = method.clone();
    impl_item.vis = syn::Visibility::Inherited;

    // Create the `execute` function. We have to do a bit of "funkiness" here because
    // we want the `self` to be the entity.
    let secret_trait_name = syn::Ident::new(
        &format!("__{}__", method.sig.ident.to_string().to_camel_case()),
        ident_span,
    );
    let execute_fn: syn::ImplItemMethod = parse_quote! {
        fn execute(db: &salsa::function::DynDb<Self>, key: Self::Key) -> Self::Value {
            trait #secret_trait_name {
                #trait_item
            }

            impl #secret_trait_name for #key_ty {
                #impl_item
            }

            <Self::Key as #secret_trait_name>::#method_name(key, db)
        }
    };

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

    let component = &args.component_ident;
    let method_ident = &method.sig.ident;

    // Find the name `db` that user gave to the second argument.
    // They can't have done any "funny business" (such as a pattern
    // like `(db, _)` or whatever) or we get an error.
    let db_var = if method.sig.inputs.len() != 2 {
        Err("method needs to have 2 arguments")
    } else {
        match &method.sig.inputs[1] {
            syn::FnArg::Receiver(_) => Err("second argment must not be self"),
            syn::FnArg::Typed(ty) => match &*ty.pat {
                syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
                _ => Err("second argment must be given a name"),
            },
        }
    };

    let block_tokens = match &db_var {
        Err(msg) => {
            let msg = proc_macro2::Literal::string(msg);
            parse_quote_spanned! { method.sig.span() =>
                {compile_error!(#msg)}
            }
        }
        Ok(db_var) => parse_quote! {
            {
                let (jar, _) = salsa::storage::HasJar::jar(#db_var);
                let component = <_ as salsa::storage::HasIngredientsFor<#component>>::ingredient(jar);
                component.#method_ident.fetch(#db_var, self)
            }
        },
    };

    // Now generate a `set_foo()` method.
    let mut set_sig = method.sig.clone();
    set_sig.ident = syn::Ident::new(
        &format!("set_{}", method.sig.ident),
        method.sig.ident.span(),
    );
    let value_ty = configuration::value_ty(&method.sig);
    set_sig.inputs.push(parse_quote! {value: #value_ty});
    set_sig.output = syn::ReturnType::Default;
    let set_block_tokens = match &db_var {
        Err(_) => parse_quote! { {} },
        Ok(db_var) => parse_quote! {
            {
                let (jar, _) = salsa::storage::HasJar::jar(#db_var);
                let component = <_ as salsa::storage::HasIngredientsFor<#component>>::ingredient(jar);
                component.#method_ident.set(#db_var, self, value)
            }
        },
    };

    (
        syn::ImplItemMethod {
            attrs: method.attrs.clone(),
            vis: method.vis.clone(),
            defaultness: method.defaultness.clone(),
            sig: method.sig.clone(),
            block: block_tokens,
        },
        syn::ImplItemMethod {
            attrs: method.attrs.clone(),
            vis: method.vis.clone(),
            defaultness: method.defaultness.clone(),
            sig: set_sig,
            block: set_block_tokens,
        },
    )
}
