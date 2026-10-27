use std::io::{Read};
use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::source_stream;
use crate::syntactic_analyzer::source_stream::SourceStream;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

pub struct Scanner<R: Read> {
    spelling: String,
    currently_scanning_token: bool,
    current_char: char,
    source_stream: SourceStream<R>,
}

impl<R: Read> Scanner<R> {
    pub fn new(mut source_stream: SourceStream<R>) -> Self {
        Self {
            spelling: String::new(),
            currently_scanning_token: false,
            current_char: source_stream.get_source(),
            source_stream,
        }
    }

    fn take_it(&mut self) {
        if self.currently_scanning_token {
            self.spelling.push(self.current_char);
        }
        self.current_char = self.source_stream.get_source();
    }

    fn scan_separator(&mut self) {
        match self.current_char {
            '!' => {
                self.take_it();
                while self.current_char != source_stream::EOL && self.current_char != source_stream::EOF {
                    self.take_it();
                }
                if self.current_char == source_stream::EOL {
                    self.take_it();
                }
            }

            ' ' | '\n' | '\r' | '\t' => {
                self.take_it();
            }

            _ => unreachable!()
        }
    }

    fn scan_token(&mut self) -> TokenKind {
        match self.current_char {
            'a'..='z' | 'A'..='Z' => {
                self.take_it();
                while Self::is_letter(self.current_char) || Self::is_digit(self.current_char) {
                    self.take_it();
                }
                TokenKind::Identifier
            }

            '0'..='9' => {
                self.take_it();
                while Self::is_digit(self.current_char) {
                    self.take_it();
                }
                TokenKind::IntLiteral
            }

            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?' => {
                self.take_it();
                while Self::is_operator(self.current_char) {
                    self.take_it();
                }
                TokenKind::Operator
            }

            '\'' => {
                self.take_it();
                self.take_it();
                if self.current_char == '\'' {
                    self.take_it();
                    TokenKind::CharLiteral
                } else {
                    TokenKind::Error
                }
            }

            '.' => {
                self.take_it();
                TokenKind::Dot
            }

            ':' => {
                self.take_it();
                if self.current_char == '=' {
                    self.take_it();
                    TokenKind::Becomes
                } else {
                    TokenKind::Colon
                }
            }

            ';' => { self.take_it(); TokenKind::Semicolon }
            ',' => { self.take_it(); TokenKind::Comma }
            '~' => { self.take_it(); TokenKind::Is }
            '(' => { self.take_it(); TokenKind::LParen }
            ')' => { self.take_it(); TokenKind::RParen }
            '[' => { self.take_it(); TokenKind::LBracket }
            ']' => { self.take_it(); TokenKind::RBracket }
            '{' => { self.take_it(); TokenKind::LCurly }
            '}' => { self.take_it(); TokenKind::RCurly }
            source_stream::EOF => TokenKind::Eof,

            _ => {
                self.take_it();
                TokenKind::Error
            }
        }
    }

    pub fn scan(&mut self) -> Token {
        self.currently_scanning_token = false;
        while matches!(self.current_char, '!' | ' ' | '\n' | '\r' | '\t') {
            self.scan_separator();
        }

        self.currently_scanning_token = true;
        self.spelling.clear();
        let start = self.source_stream.line;

        let kind = match self.scan_token() {
            TokenKind::Identifier => TokenKind::keyword_of(&self.spelling).unwrap_or(TokenKind::Identifier),
            kind => kind,
        };

        let finish = self.source_stream.line;
        Token {
            kind,
            spelling: self.spelling.clone(),
            position: SourcePosition { start, finish },
        }
    }

    fn is_letter(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z')
    }

    fn is_digit(c: char) -> bool {
        matches!(c, '0'..='9')
    }

    fn is_operator(c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?')
    }
}

pub fn scan_all(source: &[u8]) -> Vec<Token> {
    let mut scanner = Scanner::new(SourceStream::new(source));
    let mut output = vec![];

    loop {
        let token = scanner.scan();
        let is_eof = token.kind == TokenKind::Eof;
        output.push(token);

        if is_eof {
            return output;
        }
    }
}
