#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Not sure if i64 is the right type
    IntLiteral(i64),
    Identifier(String),

    // Flattened into Token
    Plus,
    Minus,
    Multiply,
    Divide,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    Equal,
    NotEqual,

    Begin,
    Const,
    Do,
    Else,
    End,
    If,
    In,
    Let,
    Then,
    Var,
    While,

    Semicolon,
    Comma,
    Becomes,
    Is,
    LParen,
    RParen,

    Eof,
}
