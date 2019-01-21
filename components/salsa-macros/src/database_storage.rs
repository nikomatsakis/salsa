use crate::parenthesized::Parenthesized;
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Peek};
use syn::{Ident, Path, Token};

/// Implementation for `salsa::database_storage!` macro.
///
/// Current syntax:
///
/// ```
///  salsa::database_storage! {
///     struct DatabaseStorage for DatabaseStruct {
///         impl HelloWorldDatabase {
///             fn input_string() for InputString;
///             fn length() for LengthQuery;
///         }
///     }
/// }
/// ```
///
/// impl Database {
pub(crate) fn database_storage(input: TokenStream) -> TokenStream {
    let DatabaseStorage {
        storage_struct_name,
        database_name,
        query_groups,
    } = syn::parse_macro_input!(input as DatabaseStorage);

    // For each query `fn foo() for FooType` create
    //
    // ```
    // foo: <FooType as ::salsa::Query<#database_name>>::Storage,
    // ```
    let mut fields = proc_macro2::TokenStream::new();
    for query_group in &query_groups {
        for Query {
            query_name,
            query_type,
        } in &query_group.queries
        {
            fields.extend(quote! {
                #query_name: <#query_type as ::salsa::Query<#database_name>>::Storage,
            });
        }
    }

    let output = quote! {
        #[derive(Default)]
        // XXX attributes
        // XXX visibility
        struct #storage_struct_name {
            #fields
        }
    };

    output.into()
}

struct DatabaseStorage {
    storage_struct_name: Ident,
    database_name: Path,
    query_groups: Vec<QueryGroup>,
}

struct QueryGroup {
    query_group: Path,
    queries: Vec<Query>,
}

struct Query {
    query_name: Ident,
    query_type: Path,
}

impl Parse for DatabaseStorage {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _struct_token: Token![struct ] = input.parse()?;
        let storage_struct_name: Ident = input.parse()?;
        let _for_token: Token![for ] = input.parse()?;
        let database_name: Path = input.parse()?;
        let content;
        syn::braced!(content in input);
        let query_groups: Vec<QueryGroup> = parse_while(Token![impl ], &content)?;
        Ok(DatabaseStorage {
            storage_struct_name,
            database_name,
            query_groups,
        })
    }
}

impl Parse for QueryGroup {
    /// ```ignore
    ///         impl HelloWorldDatabase {
    ///             fn input_string() for InputString;
    ///             fn length() for LengthQuery;
    ///         }
    /// ```
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _fn_token: Token![impl ] = input.parse()?;
        let query_group: Path = input.parse()?;
        let content;
        syn::braced!(content in input);
        let queries: Vec<Query> = parse_while(Token![fn ], &content)?;
        Ok(QueryGroup {
            query_group,
            queries,
        })
    }
}

impl Parse for Query {
    /// ```ignore
    ///             fn input_string() for InputString;
    /// ```
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _fn_token: Token![fn ] = input.parse()?;
        let query_name: Ident = input.parse()?;
        let _unit: Parenthesized<Nothing> = input.parse()?;
        let _for_token: Token![for ] = input.parse()?;
        let query_type: Path = input.parse()?;
        let _for_token: Token![;] = input.parse()?;
        Ok(Query {
            query_name,
            query_type,
        })
    }
}

struct Nothing;

impl Parse for Nothing {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(Nothing)
    }
}

fn parse_while<P: Peek + Copy, B: Parse>(peek: P, input: ParseStream) -> syn::Result<Vec<B>> {
    let mut result = vec![];
    while input.peek(peek) {
        let body: B = input.parse()?;
        result.push(body);
    }
    Ok(result)
}
