use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

pub struct TokenStream {
    tokens: Vec<Token>,
    i: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            i: 0
        }
    }
    
    pub fn next(&mut self) -> Token {
        let token = self.tokens[self.i].clone();
        
        if token.kind != TokenKind::Eof { 
            self.i += 1;
        }
        
        token
    }
}