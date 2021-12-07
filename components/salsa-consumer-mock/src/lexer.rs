use std::{iter::Peekable, sync::Arc};

#[salsa::jar(Lexer)]
pub struct LexerJar(
    text::Text,
    token_tree::TokenTree,
    token_tree::TokenTreeComponent,
    parse,
    source_text,
);

pub trait Lexer: salsa::DbWithJar<LexerJar> {}
impl<T> Lexer for T where T: salsa::DbWithJar<LexerJar> {}

pub mod text;
pub mod token_tree;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Identifier(text::Text),
    Number(text::Text),
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Tree(token_tree::TokenTree),
    Unknown(char),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    start: usize,
    end: usize,
}

#[salsa::memoized(in LexerJar)]
pub fn source_text(_db: &dyn Lexer, _filename: text::Text) -> Arc<String> {
    panic!("input")
}

#[salsa::memoized(in LexerJar)]
pub fn parse(db: &dyn Lexer, filename: text::Text) -> token_tree::TokenTree {
    let source_text = source_text(db, filename);
    let chars = &mut source_text.char_indices().peekable();
    parse_tokens(db, chars, source_text.len(), None)
}

fn parse_tokens(
    db: &dyn Lexer,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    file_len: usize,
    end_ch: Option<char>,
) -> token_tree::TokenTree {
    let mut tokens = vec![];
    let mut start_pos = file_len;
    let mut end_pos = file_len;
    while let Some((pos, ch)) = chars.peek().cloned() {
        start_pos = start_pos.min(pos);
        end_pos = end_pos.max(pos);

        if Some(ch) == end_ch {
            break;
        }

        chars.next();

        match ch {
            '(' => {
                tokens.push(Token::OpenParen);
                let tree = parse_tokens(db, chars, file_len, Some(')'));
                tokens.push(Token::Tree(tree));
            }
            ')' => {
                tokens.push(Token::CloseParen);
            }
            '[' => {
                tokens.push(Token::OpenBracket);
                let tree = parse_tokens(db, chars, file_len, Some(']'));
                tokens.push(Token::Tree(tree));
            }
            ']' => {
                tokens.push(Token::CloseBracket);
            }
            '{' => {
                tokens.push(Token::OpenBrace);
                let tree = parse_tokens(db, chars, file_len, Some('}'));
                tokens.push(Token::Tree(tree));
            }
            '}' => {
                tokens.push(Token::CloseBrace);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let text = accumulate(
                    db,
                    ch,
                    chars,
                    |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'),
                );
                tokens.push(Token::Identifier(text));
            }
            '0'..='9' => {
                let text = accumulate(db, ch, chars, |c| matches!(c, '0'..='9'));
                tokens.push(Token::Number(text));
            }
            _ => {
                tokens.push(Token::Unknown(ch));
            }
        }
    }

    token_tree::TokenTree::new(
        db,
        tokens,
        Span {
            start: start_pos,
            end: end_pos,
        },
    )
}

fn accumulate(
    db: &dyn Lexer,
    ch0: char,
    chars: &mut Peekable<impl Iterator<Item = (usize, char)>>,
    matches: impl Fn(char) -> bool,
) -> text::Text {
    let mut string = String::new();
    string.push(ch0);
    while let Some(&(_, ch1)) = chars.peek() {
        if !matches(ch1) {
            break;
        }

        string.push(ch1);
        chars.next();
    }
    text::Text::from(db, string)
}
