use syn::parse::{Parse, ParseStream};
use syn::{Ident, Path, Token};

use crate::data_item::DataItem;

// #[salsa::Entity(#id_ident in Jar0)]
// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct EntityData0 {
//    id: u32
// }

pub(crate) fn entity(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as Args);
    let data_item = syn::parse_macro_input!(input as DataItem);
    entity_contents(&args, &data_item).into()
}

pub struct Args {
    id_ident: Ident,
    _in_token: Token![in],
    jar_path: Path,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            id_ident: Parse::parse(input)?,
            _in_token: Parse::parse(input)?,
            jar_path: Parse::parse(input)?,
        })
    }
}

fn entity_contents(args: &Args, data_item: &DataItem) -> proc_macro2::TokenStream {
    let id_struct = id_struct(args);
    let id_inherent_impl = id_inherent_impl(args, data_item);
    let id_ingredients_for_impl = id_ingredients_for_impl(args, data_item);
    let id_in_db_impl = id_in_db_impl(args);
    let as_id_impl = as_id_impl(args);
    let data_inherent_impl = data_inherent_impl(args, data_item);

    quote! {
        #id_struct
        #id_inherent_impl
        #id_ingredients_for_impl
        #id_in_db_impl
        #as_id_impl

        #data_item
        #data_inherent_impl
    }
}

fn id_struct(args: &Args) -> syn::ItemStruct {
    let id_ident = &args.id_ident;
    parse_quote! {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
        pub struct #id_ident(salsa::Id);
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
                ingredients.entity_data(runtime, self)
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

fn id_in_db_impl(args: &Args) -> syn::ItemImpl {
    let Args {
        id_ident, jar_path, ..
    } = args;
    parse_quote! {
        impl<DB> salsa::entity::EntityInDb<DB> for #id_ident
        where
            DB: ?Sized + salsa::DbWithJar<#jar_path>,
        {
            fn database_key_index(self, db: &DB) -> salsa::DatabaseKeyIndex {
                let (jar, _) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#id_ident>>::ingredient(jar);
                ingredients.database_key_index(self)
            }
        }
    }
}

fn id_ingredients_for_impl(args: &Args, data_item: &DataItem) -> syn::ItemImpl {
    let Args {
        id_ident, jar_path, ..
    } = args;
    let id_data = data_item.ident();
    parse_quote! {
        impl salsa::storage::IngredientsFor for #id_ident {
            type Jar = #jar_path;
            type Ingredients = salsa::entity::EntityIngredient<#id_ident, #id_data>;

            fn create_ingredients<DB>(
                ingredients: &mut salsa::routes::Ingredients<DB>,
            ) -> Self::Ingredients
            where
                DB: salsa::storage::HasJars,
                salsa::storage::Storage<DB>: salsa::storage::HasJar<Self::Jar>,
            {
                let index = ingredients.push_mut(
                    |storage| {
                        let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar(storage);
                        <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar)
                    },
                    |storage| {
                        let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar_mut(storage);
                        <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar)
                    },
                );
                salsa::entity::EntityIngredient::new(index)
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
            pub fn new<DB: ?Sized>(self, db: &DB) -> #id_ident
            where
                DB: salsa::storage::HasJar<#jar_path>,
            {
                let (jar, runtime) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#id_ident>>::ingredient(jar);
                ingredients.new_entity(runtime, self)
            }
        }
    }
}
