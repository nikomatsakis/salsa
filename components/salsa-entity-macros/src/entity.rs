use proc_macro2::Literal;
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Field, FieldsUnnamed, Ident, ItemImpl, ItemStruct, Path, Token, VisPublic, Visibility};

// #[salsa::Entity(#entity_ident in Jar0)]
// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct EntityData0 {
//    id: u32
// }

pub(crate) fn entity(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let entity_args = syn::parse_macro_input!(args as EntityArgs);
    let entity_data_input = syn::parse_macro_input!(input as ItemStruct);
    entity_mod_and_pub_use(&entity_args, &entity_data_input).into()
}

pub struct EntityArgs {
    entity_ident: Ident,
    in_token: Token![in],
    jar_path: Path,
}

impl Parse for EntityArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            entity_ident: Parse::parse(input)?,
            in_token: Parse::parse(input)?,
            jar_path: Parse::parse(input)?,
        })
    }
}

fn entity_mod_and_pub_use(
    entity_args: &EntityArgs,
    entity_data_input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let item_mod = entity_mod(entity_args, entity_data_input);

    let mod_ident = &item_mod.ident;
    let entity_ident = &entity_args.entity_ident;
    let entity_data_ident = &entity_data_input.ident;
    let entity_data_vis = &entity_data_input.vis;

    quote! {
        #item_mod
        #entity_data_vis use #mod_ident :: #entity_ident;
        #entity_data_vis use #mod_ident :: #entity_data_ident;
    }
}

fn entity_mod(args: &EntityArgs, entity_data_input: &ItemStruct) -> syn::ItemMod {
    let mod_name = syn::Ident::new(
        &format!(
            "__{}",
            heck::SnakeCase::to_snake_case(&*args.entity_ident.to_string())
        ),
        args.entity_ident.span(),
    );

    // Create a version of the struct that is forced
    // to be public. This is a workaround for Rust's annoying
    // pub-in-private rules.
    let mut pub_entity_data = entity_data_input.clone();
    pub_entity_data.vis = Visibility::Public(VisPublic {
        pub_token: Token![pub](entity_data_input.vis.span()),
    });

    let entity_struct: ItemStruct =
        syn::parse2(entity_struct(args)).expect("entity_struct parse failed");
    let entity_inherent_impl: ItemImpl = syn::parse2(entity_inherent_impl(args, entity_data_input))
        .expect("entity_inherent_impl parse");
    let entity_ingredients_for_impl: ItemImpl =
        syn::parse2(entity_ingredients_for_impl(args, entity_data_input))
            .expect("entity_ingredients_for_impl");
    let entity_in_db_impl: ItemImpl =
        syn::parse2(entity_in_db_impl(args)).expect("entity_in_db_impl");
    let as_id_impl: ItemImpl = syn::parse2(as_id_impl(args)).expect("as_id_impl");
    let entity_data_inherent_impl: ItemImpl =
        syn::parse2(entity_data_inherent_impl(args, entity_data_input))
            .expect("entity_data_inherent_impl");

    syn::parse2(quote! {
        mod #mod_name {
            use super::*;

            #entity_struct
            #entity_inherent_impl
            #entity_ingredients_for_impl
            #entity_in_db_impl
            #as_id_impl

            #pub_entity_data
            #entity_data_inherent_impl
        }
    })
    .expect("mod")
}

fn entity_struct(args: &EntityArgs) -> proc_macro2::TokenStream {
    let entity_ident = &args.entity_ident;
    quote! {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
        pub struct #entity_ident(salsa::Id);
    }
}

fn entity_inherent_impl(
    args: &EntityArgs,
    entity_data_input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let EntityArgs {
        entity_ident,
        jar_path,
        ..
    } = args;
    let entity_data_ident = &entity_data_input.ident;
    quote! {
        impl #entity_ident {
            pub fn data<DB: ?Sized>(self, db: &DB) -> & #entity_data_ident
            where
                DB: salsa::storage::HasJar<#jar_path>,
            {
                let (jar, runtime) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor< #entity_ident >>::ingredient(jar);
                ingredients.entity_data(runtime, self)
            }
        }
    }
}

fn as_id_impl(args: &EntityArgs) -> proc_macro2::TokenStream {
    let entity_ident = &args.entity_ident;
    quote! {
        impl salsa::AsId for #entity_ident {
            fn as_id(self) -> salsa::Id {
                self.0
            }

            fn from_id(id: salsa::Id) -> Self {
                #entity_ident(id)
            }
        }

    }
}

fn entity_in_db_impl(args: &EntityArgs) -> proc_macro2::TokenStream {
    let EntityArgs {
        entity_ident,
        jar_path,
        ..
    } = args;
    quote! {
        impl<DB> salsa::entity::EntityInDb<DB> for #entity_ident
        where
            DB: ?Sized + salsa::DbWithJar<#jar_path>,
        {
            fn database_key_index(self, db: &DB) -> salsa::DatabaseKeyIndex {
                let (jar, _) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#entity_ident>>::ingredient(jar);
                ingredients.database_key_index(self)
            }
        }
    }
}

fn entity_ingredients_for_impl(
    args: &EntityArgs,
    entity_data_input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let EntityArgs {
        entity_ident,
        jar_path,
        ..
    } = args;
    let entity_data = &entity_data_input.ident;
    quote! {
        impl salsa::storage::IngredientsFor for #entity_ident {
            type Jar = #jar_path;
            type Ingredients = salsa::entity::EntityIngredient<#entity_ident, #entity_data>;

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
                        <Jar0 as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar)
                    },
                    |storage| {
                        let (jar, _) = <_ as salsa::storage::HasJar<Self::Jar>>::jar_mut(storage);
                        <Jar0 as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar)
                    },
                );
                salsa::entity::EntityIngredient::new(index)
            }
        }
    }
}

fn entity_data_inherent_impl(
    args: &EntityArgs,
    entity_data_input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let EntityArgs {
        entity_ident,
        jar_path,
        ..
    } = args;
    let entity_data_ident = &entity_data_input.ident;
    quote! {
        impl #entity_data_ident {
            pub fn new<DB: ?Sized>(self, db: &DB) -> #entity_ident
            where
                DB: salsa::storage::HasJar<#jar_path>,
            {
                let (jar, runtime) = salsa::storage::HasJar::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#entity_ident>>::ingredient(jar);
                ingredients.new_entity(runtime, self)
            }
        }
    }
}
