use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub spelling: String,
    pub position: SourcePosition,
}

impl Token {
    pub fn new(kind: TokenKind, spelling: &str, position: SourcePosition) -> Self {
        Token {
            kind: match kind {
                TokenKind::Identifier => TokenKind::get_specific_token_for_identifier(spelling),
                _ => kind,
            },
            spelling: spelling.to_string(),
            position,
        }
    }

    pub fn test_repr(&self) -> String {
        format!(
            "{},{},({}|{})",
            self.kind.spelling(),
            self.spelling.clone(),
            self.position.start,
            self.position.finish
        )
    }
}