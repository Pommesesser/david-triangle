use crate::syntactic_analyzer::token_kind::TokenKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lex: String,
    pub pos: (u64, u64),
}

impl Token {
    pub fn test_repr(&self) -> String {
        format!(
            "{},{},({}|{})",
            self.kind.spelling(),
            self.lex.clone(),
            self.pos.0,
            self.pos.1
        )
    }
}
