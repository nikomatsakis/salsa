#[salsa::jar(Parser)]
pub struct ParserJar(Token, TokenTree, TokenTreeComponent);

pub trait Parser: salsa::DbWithJar<ParserJar> {}

#[salsa::entity(TokenTree in ParserJar)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenTreeData;

#[salsa::component(TokenTreeComponent in ParserJar)]
impl TokenTree {
    fn tokens(self, _db: &dyn Parser) -> Vec<Token> {
        panic!("tokens not set")
    }
}

#[salsa::interned(Token in ParserJar)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TokenData {
    Identifier,
    Delimeted(Delimeter, TokenTree),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Delimeter {
    /// `()`
    Parens,

    /// `[]`
    Brackets,

    /// `{}`
    Braces,
}
