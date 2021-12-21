use syn::ext::IdentExt;

/// "Options" are flags that can be supplied to memoized things or entity fields.
#[derive(Default)]
pub(crate) struct Options {
    pub is_ref: Option<syn::Ident>,
    pub no_eq: Option<syn::Ident>,
}

impl Options {
    pub(crate) fn should_backdate(&self) -> bool {
        self.no_eq.is_none()
    }
}

impl syn::parse::Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut options = Options::default();

        while !input.is_empty() {
            let ident: syn::Ident = syn::Ident::parse_any(input)?;
            if ident == "ref" {
                if let Some(old) = std::mem::replace(&mut options.is_ref, Some(ident)) {
                    return Err(syn::Error::new(old.span(), "option `ref` provided twice"));
                }
            } else if ident == "no_eq" {
                if let Some(old) = std::mem::replace(&mut options.no_eq, Some(ident)) {
                    return Err(syn::Error::new(old.span(), "option `no_eq` provided twice"));
                }
            } else {
                return Err(syn::Error::new(ident.span(), "unrecognized option"));
            }
        }

        Ok(options)
    }
}
