use std::io::Read;
use crate::abstract_syntax_trees::ast::{ActualParameterSequence, Command, CommandKind, Declaration, Expression, Identifier, Program, VName};
use crate::syntactic_analyzer::scanner::Scanner;
use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::Token;
use crate::syntactic_analyzer::token_kind::TokenKind;

pub struct Parser<R: Read> {
    current_token: Token,
    lexical_analyser: Scanner<R>,
    previous_token_position: SourcePosition,
}

impl<R: Read> Parser<R> {
    pub fn new(mut scanner: Scanner<R>) -> Self {
        Self {
            current_token: scanner.scan(),
            lexical_analyser: scanner,
            previous_token_position: SourcePosition {start: 0, finish: 0},
        }
    }

    fn accept(&mut self, token_expected: TokenKind) {
        if self.current_token.kind == token_expected {
            self.previous_token_position = self.current_token.position.clone();
            self.current_token = self.lexical_analyser.scan();
        } else {
            panic!("{} expected here", token_expected.spelling())
        }
    }

    fn accept_it(&mut self) {
        self.previous_token_position = self.current_token.position.clone();
        self.current_token = self.lexical_analyser.scan();
    }

    fn start(&mut self, position: &mut SourcePosition) {
        position.start = self.current_token.position.start.clone();
    }

    fn finish(&mut self, position: &mut SourcePosition) {
        position.finish = self.previous_token_position.finish.clone();
    }

    pub fn parse_program(&mut self) -> Program {
        self.previous_token_position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.current_token = self.lexical_analyser.scan();

        let command = self.parse_command();

        Program {
            command,
            position: self.previous_token_position.clone(),
        }
    }

    fn parse_command(&mut self) -> Command {
        let mut command_position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut command_position);

        let mut command = self.parse_single_command();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            let chained_command = self.parse_single_command();
            self.finish(&mut command_position);
            command = Command {
                kind: Box::from(CommandKind::Sequential {
                    first: command,
                    second: chained_command,
                }),
                position: command_position.clone(),
            }
        }

        command
    }

    fn parse_single_command(&mut self) -> Command {
        let mut command_position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut command_position);

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);
                    self.finish(&mut command_position);

                    Command {
                        kind: Box::new(CommandKind::Call { identifier, arguments }),
                        position: command_position,
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    self.accept(TokenKind::Becomes);
                    let expression = self.parse_expression();
                    self.finish(&mut command_position);

                    Command {
                        kind: Box::new(CommandKind::Assign { v_name, expression }),
                        position: command_position,
                    }
                }
            }

            TokenKind::Begin => {
                self.accept_it();
                let command = self.parse_command();
                self.accept(TokenKind::End);
                command
            }

            TokenKind::Let => {
                self.accept_it();
                let declaration = self.parse_declaration();
                self.accept(TokenKind::In);
                let body = self.parse_single_command();
                self.finish(&mut command_position);

                Command {
                    kind: Box::new(CommandKind::Let { declaration, body }),
                    position: command_position,
                }
            }

            TokenKind::If => {
                self.accept_it();
                let condition = self.parse_expression();
                self.accept(TokenKind::Then);
                let then_branch = self.parse_single_command();
                self.accept(TokenKind::Else);
                let else_branch = self.parse_single_command();
                self.finish(&mut command_position);

                Command {
                    kind: Box::new(CommandKind::If { condition, then_branch, else_branch }),
                    position: command_position,
                }
            }

            TokenKind::While => {
                self.accept_it();
                let condition = self.parse_expression();
                self.accept(TokenKind::Do);
                let body = self.parse_single_command();
                self.finish(&mut command_position);

                Command {
                    kind: Box::new(CommandKind::While { condition, body }),
                    position: command_position,
                }
            }

            TokenKind::Semicolon | TokenKind::End | TokenKind::Else | TokenKind::In | TokenKind::Eof => {
                self.finish(&mut command_position);

                Command {
                    kind: Box::new(CommandKind::Empty),
                    position: command_position,
                }
            }

            _ => panic!("\"{}\" cannot start a command", self.current_token.spelling),
        }
    }

    fn parse_identifier(&mut self) -> Identifier {
        todo!()
    }

    fn parse_actual_parameter_sequence(&mut self) -> ActualParameterSequence {
        todo!()
    }

    fn parse_rest_of_v_name(&mut self, _identifier: Identifier) -> VName {
        todo!()
    }

    fn parse_expression(&mut self) -> Expression {
        todo!()
    }

    fn parse_declaration(&mut self) -> Declaration {
        todo!()
    }
}
