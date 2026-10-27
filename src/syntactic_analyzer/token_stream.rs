use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

pub struct TokenStream {
    tokens: std::vec::IntoIter<Token>,
    eof: Token,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        let eof = tokens.last()
            .filter(|t| t.kind == TokenKind::Eof)
            .cloned()
            .expect("token stream must end with Eof");

        Self { tokens: tokens.into_iter(), eof }
    }

    /// Hands out each token once, then repeats Eof forever like the streaming scanner did.
    pub fn next(&mut self) -> Token {
        self.tokens.next().unwrap_or_else(|| self.eof.clone())
    }
}
