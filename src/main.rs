
mod syntactic_analyzer;

use std::io::{self, BufReader};
use crate::syntactic_analyzer::scanner::Scanner;
use crate::syntactic_analyzer::source_stream::SourceStream;
use crate::syntactic_analyzer::token_kind::TokenKind;

fn main() {
    let stdin = BufReader::new(io::stdin());
    let source = SourceStream::new(stdin);
    let mut scanner = Scanner::new(source);

    loop {
        let token = scanner.scan();
        println!("{}", token.test_repr());

        if token.kind == TokenKind::Eof {
            break;
        }
    }
}
