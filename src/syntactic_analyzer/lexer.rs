use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    lex: String,
    scanning: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            lex: String::new(),
            scanning: true,
        }
    }

    fn take(&mut self) {
        if let Some(ch) = self.chars.next() {
            if self.scanning {
                self.lex.push(ch);
            }

            if ch == '\n' {
                self.line += 1;
            }
        }
    }

    fn scan_seperator(&mut self) {
        match self.chars.peek() {
            Some(_) => todo!(),
            None => todo!(),
        }
    }

    fn is_letter(c: char) -> bool {
        match c {
            'a'..'z' | 'A'..'Z' => true,
            _ => false,
        }
    }

    fn is_digit(c: char) -> bool {
        match c {
            '0'..'9' => true,
            _ => false,
        }
    }

    fn is_operator(c: char) -> bool {
        match c {
            '+' | '-' | '*' | '/' | '=' | '<' | '>' | '\\' | '&' | '@' | '%' | '^' | '?' => true,
            _ => false,
        }
    }
}
