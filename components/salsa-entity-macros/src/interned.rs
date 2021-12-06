use syn::{Ident, Path, Token};

use crate::data_item::DataItem;

// #[salsa::interned(Ty0 in Jar0)]
// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct TyData0 {
//    id: u32
// }

pub(crate) fn interned(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let data_item = syn::parse_macro_input!(input as DataItem);
    entity_mod(&args, &data_item).into()
}

pub struct Args {
    id_ident: Ident,
    _in_token: Token![in],
    jar_path: Path,
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            id_ident: syn::parse::Parse::parse(input)?,
            _in_token: syn::parse::Parse::parse(input)?,
            jar_path: syn::parse::Parse::parse(input)?,
        })
    }
}

fn entity_mod(args: &Args, data_item: &DataItem) -> proc_macro2::TokenStream {
    let interned_struct = id_struct(args);
    let id_inherent_impl = id_inherent_impl(args, data_item);
    let ingredients_for_impl = ingredients_for_impl(args, data_item);
    let as_id_impl = as_id_impl(args);
    let entity_data_inherent_impl = data_inherent_impl(args, data_item);

    quote! {
        #interned_struct
        #id_inherent_impl
        #ingredients_for_impl
        #as_id_impl

        #data_item
        #entity_data_inherent_impl
    }
}

fn id_struct(args: &Args) -> syn::ItemStruct {
    let interned_ident = &args.id_ident;
    parse_quote! {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
        pub struct #interned_ident(salsa::Id);
    }
}

fn id_inherent_impl(args: &Args, data_item: &DataItem) -> syn::ItemImpl {
    let Args {
        id_ident, jar_path, ..
    } = args;
    let data_ident = data_item.ident();
    parse_quote! {
        impl #id_ident {
            pub fn data<DB: ?Sized>(self, db: &DB) -> & #data_ident
            where
                DB: salsa::storage::HasJar<#jar_path>,
            {
                let (jar, runtime) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor< #id_ident >>::ingredient(jar);
                ingredients.data(runtime, self)
            }
        }
    }
}

fn as_id_impl(args: &Args) -> syn::ItemImpl {
    let id_ident = &args.id_ident;
    parse_quote! {
        impl salsa::AsId for #id_ident {
            fn as_id(self) -> salsa::Id {
                self.0
            }

            fn from_id(id: salsa::Id) -> Self {
                #id_ident(id)
            }
        }

    }
}

fn ingredients_for_impl(args: &Args, data_item: &DataItem) -> syn::ItemImpl {
    let Args {
        id_ident, jar_path, ..
    } = args;
    let data_ident = data_item.ident();
    parse_quote! {
        impl salsa::storage::IngredientsFor for #id_ident {
            type Jar = #jar_path;
            type Ingredients = salsa::interned::InternedIngredient<#id_ident, #data_ident>;

            fn create_ingredients<DB>(
                ingredients: &mut salsa::routes::Ingredients<DB>,
            ) -> Self::Ingredients
            where
                DB: salsa::storage::JarFromJars<Self::Jar>,
            {
                let index = ingredients.push(
                    |jars| {
                        let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                        <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar)
                    },
                );
                salsa::interned::InternedIngredient::new(index)
            }
        }
    }
}

fn data_inherent_impl(args: &Args, data_item: &DataItem) -> syn::ItemImpl {
    let Args {
        id_ident, jar_path, ..
    } = args;
    let data_ident = data_item.ident();
    parse_quote! {
        impl #data_ident {
            pub fn intern<DB: ?Sized>(self, db: &DB) -> #id_ident
            where
                DB: salsa::storage::HasJar<#jar_path>,
            {
                let (jar, runtime) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#id_ident>>::ingredient(jar);
                ingredients.intern(runtime, self)
            }
        }
    }
}
