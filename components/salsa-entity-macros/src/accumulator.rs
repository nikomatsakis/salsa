use syn::parse::{Parse, ParseStream};
use syn::{ItemStruct, Token};

// #[salsa::accumulator(in Jar0)]
// struct Accumulator(DataType);

pub(crate) fn accumulator(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let struct_impl = syn::parse_macro_input!(input as ItemStruct);
    accumulator_contents(&args, &struct_impl)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
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

fn accumulator_contents(
    args: &Args,
    struct_item: &syn::ItemStruct,
) -> syn::Result<proc_macro2::TokenStream> {
    // We expect a single anonymous field.
    let data_ty = data_ty(struct_item)?;
    let struct_name = &struct_item.ident;
    let struct_ty = &parse_quote! {#struct_name};

    let inherent_impl = inherent_impl(args, struct_ty, data_ty);
    let ingredients_for_impl = ingredients_for_impl(args, struct_ty, data_ty);
    let struct_item_out = struct_item_out(args, struct_item, data_ty);
    let accumulator_impl = accumulator_impl(args, struct_ty, data_ty);

    Ok(quote! {
        #inherent_impl
        #ingredients_for_impl
        #struct_item_out
        #accumulator_impl
    })
}

fn data_ty(struct_item: &syn::ItemStruct) -> syn::Result<&syn::Type> {
    match &struct_item.fields {
        syn::Fields::Unnamed(fields) => {
            if fields.unnamed.len() != 1 {
                return Err(syn::Error::new(
                    struct_item.ident.span(),
                    "accumulator structs should have only one anonymous field",
                ));
            } else {
                Ok(&fields.unnamed[0].ty)
            }
        }
        _ => {
            return Err(syn::Error::new(
                struct_item.ident.span(),
                "accumulator structs should have only one anonymous field",
            ));
        }
    }
}

fn struct_item_out(
    _args: &Args,
    struct_item: &syn::ItemStruct,
    data_ty: &syn::Type,
) -> syn::ItemStruct {
    let mut struct_item_out = struct_item.clone();
    struct_item_out.fields = syn::Fields::Unnamed(parse_quote! {
            (std::marker::PhantomData<#data_ty>)
    });
    struct_item_out
}

fn inherent_impl(args: &Args, struct_ty: &syn::Type, data_ty: &syn::Type) -> syn::ItemImpl {
    let jar_ty = &args.jar_ty;
    parse_quote! {
        impl #struct_ty {
            pub fn push<DB: ?Sized>(db: &DB, data: #data_ty)
            where
                DB: salsa::storage::HasJar<#jar_ty>,
            {
                let (jar, runtime) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(db);
                let ingredients = <#jar_ty as salsa::storage::HasIngredientsFor< #struct_ty >>::ingredient(jar);
                ingredients.push(runtime, data)
            }
        }
    }
}

fn ingredients_for_impl(args: &Args, struct_ty: &syn::Type, data_ty: &syn::Type) -> syn::ItemImpl {
    let jar_ty = &args.jar_ty;
    parse_quote! {
        impl salsa::storage::IngredientsFor for #struct_ty {
            type Ingredients = salsa::accumulator::AccumulatorIngredient<#data_ty>;
            type Jar = #jar_ty;

            fn create_ingredients<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self::Ingredients
            where
                DB: salsa::DbWithJar<Self::Jar> + salsa::storage::JarFromJars<Self::Jar>,
            {
                    let index = ingredients.push_mut(
                        |jars| {
                            let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                            <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar)
                        },
                        |jars| {
                            let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars_mut(jars);
                            <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar)
                        },
                    );
                    salsa::accumulator::AccumulatorIngredient::new(index)
            }
        }
    }
}

fn accumulator_impl(args: &Args, struct_ty: &syn::Type, data_ty: &syn::Type) -> syn::ItemImpl {
    let jar_ty = &args.jar_ty;
    parse_quote! {
        impl salsa::accumulator::Accumulator for #struct_ty {
            type Data = #data_ty;
            type Jar = #jar_ty;

            fn accumulator_ingredient<'db, Db>(
                db: &'db Db,
            ) -> &'db salsa::accumulator::AccumulatorIngredient<Self::Data>
            where
                Db: ?Sized + salsa::storage::HasJar<Self::Jar>
            {
                let (jar, _) = <_ as salsa::storage::HasJar<#jar_ty>>::jar(db);
                let ingredients = <#jar_ty as salsa::storage::HasIngredientsFor<#struct_ty>>::ingredient(jar);
                ingredients
            }
        }
    }
}
