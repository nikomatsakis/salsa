use heck::{CamelCase, TitleCase};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Ident, ImplItemMethod, ItemImpl, ItemStruct, Path, Token};

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
    jar_path: Path,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        Ok(Self {
            component_ident: Parse::parse(input)?,
            _in_token: Parse::parse(input)?,
            jar_path: Parse::parse(input)?,
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
) -> (Vec<syn::ImplItem>, Vec<syn::ItemStruct>) {
    let mut impl_items = vec![];
    let mut item_structs = vec![];
    let mut fields = vec![];

    for impl_item in &item_impl.items {
        match impl_item {
            syn::ImplItem::Const(_) | syn::ImplItem::Type(_) => impl_items.push(impl_item.clone()),

            syn::ImplItem::Method(m) => {
                let method_name = &m.sig.ident;

                // Generate a struct to "house" the configuration for this method.
                let struct_name = syn::Ident::new(
                    &format!("{}{}", args.component_ident, method_name).to_camel_case(),
                    method_name.span(),
                );
                item_structs.push(
                    syn::parse2(quote! {
                        struct #struct_name;
                    })
                    .unwrap(),
                );

                impl_items.push(impl_item.clone());
            }

            _ => {
                impl_items.push(syn::parse2(quote_spanned! {
                    impl_item.span() => compile_error!("only constants, methods, and types are permitted")
                }).unwrap());
            }
        }
    }

    (impl_items, item_structs, fields)
}

fn component_struct(args: &Args) -> proc_macro2::TokenStream {
    let component_ident = &args.component_ident;
    quote! {
        pub struct #component_ident {
            method: salsa::function::FunctionIngredient<EntityComponent0_method>,
        }
    }
}
