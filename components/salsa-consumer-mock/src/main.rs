use std::sync::Arc;

mod db;
mod error;
mod jar0;
mod lexer;

fn main() {
    let mut db = db::TheDatabase::default();

    let dummy_rs = lexer::text::Text::from(&db, format!("dummy.rs"));
    lexer::source_text::set(&mut db, dummy_rs, Arc::new(format!("foo(bar, baz[3])) ~")));
    let ast = lexer::lex(&db, dummy_rs);
    print(&db, 0, ast);
    let diagnostics = lexer::lex::accumulated::<crate::error::Diagnostics>(&db, dummy_rs);
    eprintln!("{:?}", diagnostics);
}

fn print(db: &dyn lexer::Lexer, indent: usize, tree: lexer::token_tree::TokenTree) {
    let tokens = tree.tokens(db);
    for token in tokens {
        for _ in 0..indent {
            print!(" ");
        }
        print!("token = {:?}", token);
        match token {
            lexer::Token::Identifier(text) => {
                println!(" = {:?}", text.data(db));
            }
            lexer::Token::Number(text) => {
                println!(" = {:?}", text.data(db));
            }
            lexer::Token::Tree(tree) => {
                println!();
                print(db, indent + 2, *tree);
            }
            _ => println!(),
        }
    }
}
