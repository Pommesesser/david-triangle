use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

const EOF: char = '\0';
const EOL: char = '\n';

pub struct Scanner<'a> {
    curr_char: char,
    curr_i: usize,

    src: &'a [u8],
    src_i: usize,
    src_line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        let mut scanner = Self {
            curr_char: EOF,
            curr_i: 0,
            src: source,
            src_i: 0,
            src_line: 1,
        };
        scanner.curr_char = scanner.get_source();
        scanner
    }

    /// Each byte is one char (Latin-1), and past the end is EOF, like Java's SourceFile.
    /// The line is counted when the newline is fetched, not when it is consumed.
    fn get_source(&mut self) -> char {
        let Some(&byte) = self.src.get(self.src_i) else {
            return EOF;
        };
        self.src_i += 1;

        let c = char::from(byte);
        if c == EOL {
            self.src_line += 1;
        }
        c
    }

    fn take_it(&mut self) {
        self.curr_char = self.get_source();
        if self.curr_i < self.src.len() {
            self.curr_i += 1;
        }
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

            _ => unreachable!(),
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

            ';' => {
                self.take_it();
                TokenKind::Semicolon
            }
            ',' => {
                self.take_it();
                TokenKind::Comma
            }
            '~' => {
                self.take_it();
                TokenKind::Is
            }
            '(' => {
                self.take_it();
                TokenKind::LParen
            }
            ')' => {
                self.take_it();
                TokenKind::RParen
            }
            '[' => {
                self.take_it();
                TokenKind::LBracket
            }
            ']' => {
                self.take_it();
                TokenKind::RBracket
            }
            '{' => {
                self.take_it();
                TokenKind::LCurly
            }
            '}' => {
                self.take_it();
                TokenKind::RCurly
            }
            EOF => TokenKind::Eof,

            _ => {
                self.take_it();
                TokenKind::Error
            }
        }
    }

    /// Scans the next token; at the end it keeps returning the same Eof token.
    pub fn scan(&mut self) -> Token<'a> {
        while matches!(self.curr_char, '!' | ' ' | '\n' | '\r' | '\t') {
            self.scan_separator();
        }

        let spelling_start = self.curr_i;
        let line_start = self.src_line;

        let kind = self.scan_token();

        let spelling_finish = self.curr_i;
        let line_finish = self.src_line;

        let spelling = &self.src[spelling_start..spelling_finish];
        let kind = match kind {
            TokenKind::Identifier => {
                TokenKind::keyword_of(spelling).unwrap_or(TokenKind::Identifier)
            }
            kind => kind,
        };

        Token {
            kind,
            spelling,
            position: SourcePosition {
                start: line_start,
                finish: line_finish,
            },
        }
    }

    fn is_letter(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z')
    }

    fn is_digit(c: char) -> bool {
        matches!(c, '0'..='9')
    }

    fn is_operator(c: char) -> bool {
        matches!(
            c,
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?'
        )
    }
}

pub fn scan_all<'a>(source: &'a [u8]) -> Vec<Token<'a>> {
    let mut scanner = Scanner::new(source);
    let mut output = Vec::with_capacity(source.len() / 4 + 1);

    loop {
        let token = scanner.scan();
        let is_eof = token.kind == TokenKind::Eof;
        output.push(token);

        if is_eof {
            return output;
        }
    }
}
