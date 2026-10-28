use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

const EOF: char = '\0';
const EOL: char = '\n';

struct Scanner<'a> {
    spelling: String,
    curr_scanning: bool,
    curr_char: char,
    src: &'a [u8],
    idx: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a [u8]) -> Self {
        let mut scanner = Self {
            spelling: String::new(),
            curr_scanning: false,
            curr_char: EOF,
            src: source,
            idx: 0,
            line: 1,
        };
        scanner.curr_char = scanner.get_source();
        scanner
    }

    /// Each byte is one char (Latin-1), and past the end is EOF, like Java's SourceFile.
    /// The line is counted when the newline is fetched, not when it is consumed.
    fn get_source(&mut self) -> char {
        let Some(&byte) = self.src.get(self.idx) else {
            return EOF;
        };
        self.idx += 1;

        let c = char::from(byte);
        if c == EOL {
            self.line += 1;
        }
        c
    }

    fn take_it(&mut self) {
        if self.curr_scanning {
            self.spelling.push(self.curr_char);
        }
        self.curr_char = self.get_source();
    }

    fn scan_separator(&mut self) {
        match self.curr_char {
            '!' => {
                self.take_it();
                while self.curr_char != EOL && self.curr_char != EOF {
                    self.take_it();
                }
                if self.curr_char == EOL {
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
        match self.curr_char {
            'a'..='z' | 'A'..='Z' => {
                self.take_it();
                while Self::is_letter(self.curr_char) || Self::is_digit(self.curr_char) {
                    self.take_it();
                }
                TokenKind::Identifier
            }

            '0'..='9' => {
                self.take_it();
                while Self::is_digit(self.curr_char) {
                    self.take_it();
                }
                TokenKind::IntLiteral
            }

            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?' => {
                self.take_it();
                while Self::is_operator(self.curr_char) {
                    self.take_it();
                }
                TokenKind::Operator
            }

            '\'' => {
                self.take_it();
                self.take_it();
                if self.curr_char == '\'' {
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
                if self.curr_char == '=' {
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
            EOF => TokenKind::Eof,

            _ => {
                self.take_it();
                TokenKind::Error
            }
        }
    }

    fn scan(&mut self) -> Token {
        self.curr_scanning = false;
        while matches!(self.curr_char, '!' | ' ' | '\n' | '\r' | '\t') {
            self.scan_separator();
        }

        self.curr_scanning = true;
        self.spelling.clear();

        let start = self.line;
        let kind = match self.scan_token() {
            TokenKind::Identifier => TokenKind::keyword_of(&self.spelling).unwrap_or(TokenKind::Identifier),
            kind => kind,
        };
        let finish = self.line;

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
    let mut scanner = Scanner::new(source);
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
