use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub spelling: String,
    pub position: SourcePosition,
}
