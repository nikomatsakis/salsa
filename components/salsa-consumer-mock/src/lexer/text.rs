use super::{Lexer, LexerJar};

#[salsa::interned(Text in LexerJar)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextData {
    pub string: String,
}

impl Text {
    pub fn from(db: &dyn Lexer, string: String) -> Self {
        TextData { string }.intern(db)
    }
}
