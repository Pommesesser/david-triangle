use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub position: SourcePosition,
    pub spelling: &'a [u8],
}

/// Each byte is one char (Latin-1), the same way the scanner reads the source.
pub fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}
