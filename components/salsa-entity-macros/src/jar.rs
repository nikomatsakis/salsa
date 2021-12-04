use proc_macro2::Literal;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Field, FieldsUnnamed, Ident, ItemStruct, Path, Token};

// Source:
//
// #[salsa::jar(Jar0Db)]
// pub struct Jar0(Entity0, Ty0, EntityComponent0, my_func);

pub(crate) fn jar(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let jar_trait = syn::parse_macro_input!(args as Path);
    let input = syn::parse_macro_input!(input as ItemStruct);
    jar_struct_and_friends(&jar_trait, &input).into()
}

pub(crate) fn jar_struct_and_friends(
    jar_trait: &Path,
    input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let output_struct = jar_struct(input);

    let jar_struct = &input.ident;

    // for each field, we need to generate an impl of `HasIngredientsFor`
    let has_ingredients_for_impls: Vec<_> = input
        .fields
        .iter()
        .zip(0..)
        .map(|(field, index)| has_ingredients_for_impl(jar_struct, field, index))
        .collect();

    let jar_impl = jar_impl(jar_struct, jar_trait, input);

    quote! {
        #output_struct

        #(#has_ingredients_for_impls)*

        #jar_impl
    }
}

pub(crate) fn has_ingredients_for_impl(
    jar_struct: &Ident,
    field: &Field,
    index: u32,
) -> proc_macro2::TokenStream {
    let field_ty = &field.ty;
    let index = Literal::u32_unsuffixed(index);
    quote! {
        impl salsa::storage::HasIngredientsFor<#field_ty> for #jar_struct {
            fn ingredient(&self) -> &<#field_ty as IngredientsFor>::Ingredients {
                &self.#index
            }

            fn ingredient_mut(&mut self) -> &mut <#field_ty as IngredientsFor>::Ingredients {
                &mut self.#index
            }
        }
    }
}

pub(crate) fn jar_impl(
    jar_struct: &Ident,
    jar_trait: &Path,
    input: &ItemStruct,
) -> proc_macro2::TokenStream {
    let field_tys: Vec<_> = input.fields.iter().map(|f| &f.ty).collect();
    let field_var_names: &Vec<_> = &input
        .fields
        .iter()
        .zip(0..)
        .map(|(f, i)| Ident::new(&format!("i{}", i), f.ty.span()))
        .collect();
    quote! {
        impl<'salsa_db> salsa::jar::Jar<'salsa_db> for #jar_struct {
            type DynDb = dyn #jar_trait + 'salsa_db;

            fn create_jar<DB>(ingredients: &mut salsa::routes::Ingredients<DB>) -> Self
            where
                DB: salsa::storage::HasJars + salsa::storage::DbWithJar<Self>,
                salsa::storage::Storage<DB>: salsa::storage::HasJar<Self>,
            {
                #(
                    let #field_var_names = <#field_tys as IngredientsFor>::create_ingredients(ingredients);
                )*
                Self(#(#field_var_names),*)
            }
        }
    }
}

pub(crate) fn jar_struct(input: &ItemStruct) -> ItemStruct {
    let mut output_struct = input.clone();
    output_struct.fields = generate_fields(&input).into();
    if output_struct.semi_token.is_none() {
        output_struct.semi_token = Some(Token![;](input.struct_token.span));
    }
    output_struct
}

fn generate_fields(input: &ItemStruct) -> FieldsUnnamed {
    // Generate the
    let mut output_fields = Punctuated::new();
    for field in input.fields.iter() {
        let mut field = field.clone();

        // Convert to anonymous fields
        field.ident = None;

        let field_ty = &field.ty;
        field.ty =
            syn::parse2(quote!(< #field_ty as salsa::storage::IngredientsFor >::Ingredients))
                .unwrap();

        output_fields.push(field);
    }

    let paren_token = match &input.fields {
        syn::Fields::Named(f) => syn::token::Paren {
            span: f.brace_token.span,
        },
        syn::Fields::Unnamed(f) => f.paren_token,
        syn::Fields::Unit => syn::token::Paren {
            span: input.ident.span(),
        },
    };

    FieldsUnnamed {
        paren_token,
        unnamed: output_fields,
    }
}
