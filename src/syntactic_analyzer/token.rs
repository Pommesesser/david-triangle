use crate::syntactic_analyzer::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lex: String,
    pub pos: (u64, u64),
}
