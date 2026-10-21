use std::str::Chars;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

const EOT: char = '\0';

pub struct Lexer<'a> {
    chars: Chars<'a>,
    current_char: char,
    line: u64,
    lex: String,
    currently_scanning_token: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let current_char = chars.next().unwrap_or(EOT);

        Self {
            chars,
            current_char,
            line: 1,
            lex: String::new(),
            currently_scanning_token: true,
        }
    }

    fn take(&mut self) {
        if self.currently_scanning_token {
            self.lex.push(self.current_char);
        }
        if self.current_char == '\n' {
            self.line += 1;
        }
        self.current_char = self.chars.next().unwrap_or(EOT);
    }

    fn lex_separator(&mut self) {
        match self.current_char {
            '!' => {
                self.take();
                while self.current_char != '\n' && self.current_char != EOT {
                    self.take();
                }
                if self.current_char == '\n' {
                    self.take();
                }
            }
            ' ' | '\n' | '\r' | '\t' => {
                self.take();
            }
            _ => {}
        }
    }

    fn lex_token(&mut self) -> TokenKind {
        match self.current_char {
            'a'..='z' | 'A'..='Z' => {
                self.take();
                while Self::is_letter(self.current_char) || Self::is_digit(self.current_char) {
                    self.take();
                }
                TokenKind::Identifier
            }
            '0'..='9' => {
                self.take();
                while Self::is_digit(self.current_char) {
                    self.take();
                }
                TokenKind::IntLiteral
            }
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?' => {
                self.take();
                while Self::is_operator(self.current_char) {
                    self.take();
                }
                TokenKind::Operator
            }
            '\'' => {
                self.take();
                self.take();
                if self.current_char == '\'' {
                    self.take();
                    TokenKind::CharLiteral
                } else {
                    TokenKind::Error
                }
            }
            '.' => { self.take(); TokenKind::Dot }
            ':' => {
                self.take();
                if self.current_char == '=' {
                    self.take();
                    TokenKind::Becomes
                } else {
                    TokenKind::Colon
                }
            }
            ';' => { self.take(); TokenKind::Semicolon }
            ',' => { self.take(); TokenKind::Comma }
            '~' => { self.take(); TokenKind::Is }
            '(' => { self.take(); TokenKind::LParen }
            ')' => { self.take(); TokenKind::RParen }
            '[' => { self.take(); TokenKind::LBracket }
            ']' => { self.take(); TokenKind::RBracket }
            '{' => { self.take(); TokenKind::LCurly }
            '}' => { self.take(); TokenKind::RCurly }
            EOT => TokenKind::Eof,
            _ => {
                self.take();
                TokenKind::Error
            }
        }
    }

    pub fn lex(&mut self) -> Token {
        self.currently_scanning_token = false;
        while matches!(self.current_char, '!' | ' ' | '\n' | '\r' | '\t') {
            self.lex_separator();
        }

        self.currently_scanning_token = true;
        self.lex.clear();

        let start = self.line;
        let kind = self.lex_token();
        let finish = self.line;

        Token {
            kind,
            lex: self.lex.clone(),
            pos: (start, finish),
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
