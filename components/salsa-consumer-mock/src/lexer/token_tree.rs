use super::{Lexer, LexerJar, Span, Token};

#[salsa::entity(TokenTree in LexerJar)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenTreeData(Private);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct Private;

#[salsa::component(TokenTreeComponent in LexerJar)]
impl TokenTree {
    pub fn tokens(self, _db: &dyn Lexer) -> Vec<Token> {
        unreachable!()
    }

    pub fn span(self, _db: &dyn Lexer) -> Span {
        unreachable!()
    }
}

impl TokenTree {
    pub fn new(db: &dyn Lexer, tokens: Vec<Token>, span: Span) -> Self {
        let result = TokenTreeData(Private).new(db);
        result.set_tokens(db, tokens);
        result.set_span(db, span);
        result
    }
}
