use heck::CamelCase;
use proc_macro2::Literal;

// salsa::entity! {
//     entity TokenTree in LexerJar {
//         #[id] name: String,
//         tokens: Vec<Token>,
//         span: Span,
//     }
// }

pub(crate) fn entity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let entity = syn::parse_macro_input!(input as Entity);
    entity_contents(&entity).into()
}

mod kw {
    syn::custom_keyword![entity];
}

pub struct Entity {
    _entity: kw::entity,
    ident: syn::Ident,
    _in_token: syn::Token![in],
    jar_path: syn::Path,
    fields: syn::FieldsNamed,
}

impl syn::parse::Parse for Entity {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            _entity: syn::parse::Parse::parse(input)?,
            ident: syn::parse::Parse::parse(input)?,
            _in_token: syn::parse::Parse::parse(input)?,
            jar_path: syn::parse::Parse::parse(input)?,
            fields: syn::parse::Parse::parse(input)?,
        })
    }
}

impl Entity {
    fn is_id_field(field: &syn::Field) -> bool {
        field.attrs.iter()
        .any(|a| a.path.is_ident("id"))
    }

    fn other_fields_count(&self) -> usize {
        self.fields.named.iter().filter(|f| !Self::is_id_field(f)).count()
    }

    /// For the entity, we create a tuple that contains the function ingredients
    /// for each "other" field and the entity ingredient. This is the index of
    /// the entity ingredient within that tuple.
    fn entity_index(&self) -> Literal {
        Literal::usize_unsuffixed(self.other_fields_count())
    }

    /// For the entity, we create a tuple that contains the function ingredients
    /// for each "other" field and the entity ingredient. These are the indices
    /// of the function ingredients within that tuple.
    fn other_field_indices(&self) -> Vec<Literal> {
        (0..self.other_fields_count())
        .map(|i| Literal::usize_unsuffixed(i))
        .collect()
    }

    fn all_field_names(&self) -> Vec<&syn::Ident> {
        self.fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap())
            .collect()
    }

    fn all_field_tys(&self) -> Vec<&syn::Type> {
        self.fields.named.iter().map(|f| &f.ty).collect()
    }

    fn id_field_names(&self) -> Vec<&syn::Ident> {
        self.fields
            .named
            .iter()
            .filter(|f| Self::is_id_field(f))
            .map(|f| f.ident.as_ref().unwrap())
            .collect()
    }

    fn id_field_tys(&self) -> Vec<&syn::Type> {
        self.fields
            .named
            .iter()
            .filter(|f| Self::is_id_field(f))
            .map(|f| &f.ty)
            .collect()
    }

    fn other_field_names(&self) -> Vec<&syn::Ident> {
        self.fields
            .named
            .iter()
            .filter(|f| !Self::is_id_field(f))
            .map(|f| f.ident.as_ref().unwrap())
            .collect()
    }

    fn other_field_tys(&self) -> Vec<&syn::Type> {
        self.fields
            .named
            .iter()
            .filter(|f| !Self::is_id_field(f))
            .map(|f| &f.ty)
            .collect()
    }
}

fn entity_contents(entity: &Entity) -> proc_macro2::TokenStream {
    let id_struct = id_struct(entity);
    let config_structs = config_structs(entity);
    let id_inherent_impl = id_inherent_impl(entity);
    let id_ingredients_for_impl = id_ingredients_for_impl(entity, &config_structs);
    let id_in_db_impl = id_in_db_impl(entity);
    let as_id_impl = as_id_impl(entity);
    let config_impls = config_impls(entity, &config_structs);

    quote! {
        #id_struct
        #id_inherent_impl
        #id_ingredients_for_impl
        #id_in_db_impl
        #as_id_impl
        #(#config_structs)*
        #(#config_impls)*
    }
}

fn id_struct(entity: &Entity) -> syn::ItemStruct {
    let ident = &entity.ident;
    parse_quote! {
        #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
        pub struct #ident(salsa::Id);
    }
}

fn config_structs(entity: &Entity) -> Vec<syn::ItemStruct> {
    let ident = &entity.ident;
    let other_field_names = entity.other_field_names();
    other_field_names
    .iter()
    .map(|other_field_name| {
        let config_name = syn::Ident::new(
            &format!("__{}", format!("{}_{}", ident, other_field_name).to_camel_case()),
            other_field_name.span(),
        );
        parse_quote! {
            #[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug)]
            pub struct #config_name(std::convert::Infallible);
        }
    })
    .collect()
}

fn id_inherent_impl(entity: &Entity) -> syn::ItemImpl {
    let Entity {
        ident, jar_path, ..
    } = entity;
    let all_field_names = entity.all_field_names();
    let all_field_tys = entity.all_field_tys();
    let id_field_names = entity.id_field_names();
    let id_field_indices: Vec<_> = (0..id_field_names.len())
        .map(|i| Literal::usize_unsuffixed(i))
        .collect();
    let id_field_tys = entity.id_field_tys();
    let other_field_names = entity.other_field_names();
    let other_field_indices = entity.other_field_indices();
    let other_field_tys = entity.other_field_tys();
    let entity_index = entity.entity_index();

    // FIXME: It'd be nicer to make a DB parameter, but only because dyn upcasting doesn't work.
    // Making DB a *parameter* would work except that 
    let db_dyn_ty: syn::Type = parse_quote! {
        <#jar_path as salsa::jar::Jar<'_>>::DynDb
    };

    parse_quote! {
        impl #ident {
            pub fn new(__db: &#db_dyn_ty, #(#all_field_names: #all_field_tys,)*) -> Self
            {
                let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_path>>::jar(__db);
                let __ingredients = <#jar_path as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                let __id = __ingredients.#entity_index.new_entity(__runtime, (#(#id_field_names,)*));
                #(
                    __ingredients.#other_field_indices.set(__db, __id, #other_field_names);
                )*
                __id
            }

            #(
                pub fn #id_field_names<'db>(self, __db: &'db #db_dyn_ty) -> &'db #id_field_tys
                {
                    let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_path>>::jar(__db);
                    let __ingredients = <#jar_path as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                    &__ingredients.#entity_index.entity_data(__runtime, self).#id_field_indices
                }
            )*

            #(
                pub fn #other_field_names<'db>(self, __db: &'db #db_dyn_ty) -> &'db #other_field_tys {
                    let (__jar, __runtime) = <_ as salsa::storage::HasJar<#jar_path>>::jar(__db);
                    let __ingredients = <#jar_path as salsa::storage::HasIngredientsFor< #ident >>::ingredient(__jar);
                    __ingredients.#other_field_indices.fetch(__db, self)
                }
            )*
        }
    }
}

fn config_impls(entity: &Entity, config_structs: &[syn::ItemStruct]) -> Vec<syn::ItemImpl> {
    let Entity {
        ident, jar_path, ..
    } = entity;
    let other_field_tys = entity.other_field_tys();
    other_field_tys
    .into_iter()
    .zip(config_structs.iter().map(|s| &s.ident))
    .map(|( other_field_ty, config_struct_name)| {
        parse_quote! {
            impl salsa::function::Configuration for #config_struct_name {
                type Jar = #jar_path;
                type Key = #ident;
                type Value = #other_field_ty;
                const CYCLE_STRATEGY: salsa::cycle::CycleRecoveryStrategy = salsa::cycle::CycleRecoveryStrategy::Panic;

                fn should_backdate_value(old_value: &Self::Value, new_value: &Self::Value) -> bool {
                    salsa::function::should_backdate_value(old_value, new_value)
                }

                fn execute(db: &salsa::function::DynDb<Self>, key: Self::Key) -> Self::Value {
                    unreachable!()
                }
            
                fn recover_from_cycle(db: &salsa::function::DynDb<Self>, cycle: &salsa::Cycle, key: Self::Key) -> Self::Value {
                    unreachable!()
                }
            }
        }
    })
    .collect()
}

fn as_id_impl(entity: &Entity) -> syn::ItemImpl {
    let ident = &entity.ident;
    parse_quote! {
        impl salsa::AsId for #ident {
            fn as_id(self) -> salsa::Id {
                self.0
            }

            fn from_id(id: salsa::Id) -> Self {
                #ident(id)
            }
        }
    }
}

fn id_in_db_impl(entity: &Entity) -> syn::ItemImpl {
    let Entity {
        ident, jar_path, ..
    } = entity;
    let entity_index = entity.entity_index();
    parse_quote! {
        impl<DB> salsa::entity::EntityInDb<DB> for #ident
        where
            DB: ?Sized + salsa::DbWithJar<#jar_path>,
        {
            fn database_key_index(self, db: &DB) -> salsa::DatabaseKeyIndex {
                let (jar, _) = <_ as salsa::storage::HasJar<#jar_path>>::jar(db);
                let ingredients = <#jar_path as salsa::storage::HasIngredientsFor<#ident>>::ingredient(jar);
                ingredients.#entity_index.database_key_index(self)
            }
        }
    }
}

fn id_ingredients_for_impl(entity: &Entity, config_structs: &[syn::ItemStruct]) -> syn::ItemImpl {
    let Entity {
        ident, jar_path, ..
    } = entity;
    let id_field_tys = entity.id_field_tys();
    let other_field_indices: Vec<_> = entity.other_field_indices();
    let entity_index = entity.entity_index();
    let config_struct_names = config_structs.iter().map(|s| &s.ident);
    parse_quote! {
        impl salsa::storage::IngredientsFor for #ident {
            type Jar = #jar_path;
            type Ingredients = (
                #(
                    salsa::function::FunctionIngredient<#config_struct_names>,
                )*
                salsa::entity::EntityIngredient<#ident, (#(#id_field_tys,)*)>,
            );

            fn create_ingredients<DB>(
                ingredients: &mut salsa::routes::Ingredients<DB>,
            ) -> Self::Ingredients
            where
                DB: salsa::DbWithJar<Self::Jar> + salsa::storage::JarFromJars<Self::Jar>,
            {
                (
                    #(
                        {
                            let index = ingredients.push(
                                |jars| {
                                    let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                                    let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar);
                                    &ingredients.#other_field_indices
                                },
                            );
                            salsa::function::FunctionIngredient::new(index)
                        },
                    )*
                    {
                        let index = ingredients.push_mut(
                            |jars| {
                                let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars(jars);
                                let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient(jar);
                                &ingredients.#entity_index
                            },
                            |jars| {
                                let jar = <DB as salsa::storage::JarFromJars<Self::Jar>>::jar_from_jars_mut(jars);
                                let ingredients = <_ as salsa::storage::HasIngredientsFor<Self>>::ingredient_mut(jar);
                                &mut ingredients.#entity_index
                            },
                        );
                        salsa::entity::EntityIngredient::new(index)
                    },
                )
            }
        }
    }
}
