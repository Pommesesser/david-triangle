use std::io::Read;
use crate::syntactic_analyzer::scanner::Scanner;
use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

pub struct Parser<R: Read> {
    current_token: Token,
    lexical_analyser: Scanner<R>,
    previous_token_position: SourcePosition,
}

impl<R: Read> Parser<R> {
    pub fn new(mut scanner: Scanner<R>) -> Self {
        Self {
            current_token: scanner.scan(),
            lexical_analyser: scanner,
            previous_token_position: SourcePosition {start: 0, finish: 0},
        }
    }

    fn accept(&mut self, token_expected: TokenKind) {
        if self.current_token.kind == token_expected {
            self.previous_token_position = self.current_token.position.clone();
            self.current_token = self.lexical_analyser.scan();
        } else {
            panic!("{} expected here", token_expected.spelling())
        }
    }

    fn accept_it(&mut self) {
        self.previous_token_position = self.current_token.position.clone();
        self.current_token = self.lexical_analyser.scan();
    }

    fn start(&mut self, position: &mut SourcePosition) {
        position.start = self.previous_token_position.start.clone();
    }

    fn finish(&mut self, position: &mut SourcePosition) {
        position.finish = self.previous_token_position.finish.clone();
    }

    pub fn parse_program()  {

    }
}