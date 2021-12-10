use super::{LexerJar, Span, Token};

salsa::entity2! {
    entity TokenTree in LexerJar {
        tokens: Vec<Token>,
        #[clone] span: Span,
    }
}
