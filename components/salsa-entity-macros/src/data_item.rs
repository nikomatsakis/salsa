pub(crate) enum DataItem {
    Struct(syn::ItemStruct),
    Enum(syn::ItemEnum),
}

impl syn::parse::Parse for DataItem {
    fn parse(input: &syn::parse::ParseBuffer<'_>) -> Result<Self, syn::Error> {
        match syn::Item::parse(input)? {
            syn::Item::Enum(item) => Ok(DataItem::Enum(item)),
            syn::Item::Struct(item) => Ok(DataItem::Struct(item)),
            _ => Err(input.error("expected an enum or a struct")),
        }
    }
}

impl DataItem {
    pub fn ident(&self) -> &syn::Ident {
        match self {
            DataItem::Struct(s) => &s.ident,
            DataItem::Enum(e) => &e.ident,
        }
    }
}

impl quote::ToTokens for DataItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DataItem::Struct(s) => s.to_tokens(tokens),
            DataItem::Enum(e) => e.to_tokens(tokens),
        }
    }
}
