pub(crate) struct Parenthesized<T>(pub T);

impl<T> syn::parse::Parse for Parenthesized<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        content.parse::<T>().map(Parenthesized)
    }
}
