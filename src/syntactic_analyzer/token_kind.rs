#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    IntLiteral,
    CharLiteral,
    Identifier,
    Operator,

    Array,
    Begin,
    Const,
    Do,
    Else,
    End,
    Func,
    If,
    In,
    Let,
    Of,
    Proc,
    Record,
    Then,
    Type,
    Var,
    While,

    Dot,
    Colon,
    Semicolon,
    Comma,
    Becomes,
    Is,

    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,

    Eof,
    Error,
}

impl TokenKind {
    pub fn of(lex: &str) -> TokenKind {
        match lex {
            "array" => TokenKind::Array,
            "begin" => TokenKind::Begin,
            "const" => TokenKind::Const,
            "do" => TokenKind::Do,
            "else" => TokenKind::Else,
            "end" => TokenKind::End,
            "func" => TokenKind::Func,
            "if" => TokenKind::If,
            "in" => TokenKind::In,
            "let" => TokenKind::Let,
            "of" => TokenKind::Of,
            "proc" => TokenKind::Proc,
            "record" => TokenKind::Record,
            "then" => TokenKind::Then,
            "type" => TokenKind::Type,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        }
    }
}