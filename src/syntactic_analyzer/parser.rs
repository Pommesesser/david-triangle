use std::io::Read;
use crate::abstract_syntax_trees::ast::{ActualParameter, ActualParameterKind, ActualParameterSequence, ActualParameterSequenceKind, ArrayAggregate, ArrayAggregateKind, CharacterLiteral, Command, CommandKind, Declaration, DeclarationKind, Expression, ExpressionKind, FieldTypeDenoter, FieldTypeDenoterKind, FormalParameter, FormalParameterKind, FormalParameterSequence, FormalParameterSequenceKind, Identifier, IntegerLiteral, Operator, Program, RecordAggregate, RecordAggregateKind, TypeDenoter, TypeDenoterKind, VName, VNameKind};
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
            self.syntactic_error(&format!("\"{}\" expected here", token_expected.spelling()))
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

    // Same output as the Java ErrorReporter, but aborts instead of throwing SyntaxError.
    fn syntactic_error(&self, message: &str) -> ! {
        let position = &self.current_token.position;
        panic!("ERROR: {} {}..{}", message, position.start, position.finish)
    }

    pub fn parse_program(&mut self) -> Program {
        // The first token was already scanned by `new`.
        self.previous_token_position = SourcePosition {
            start: 0,
            finish: 0,
        };

        let command = self.parse_command();
        let program = Program {
            command,
            position: self.previous_token_position.clone(),
        };
        if self.current_token.kind != TokenKind::Eof {
            self.syntactic_error(&format!("\"{}\" not expected after end of program", self.current_token.spelling));
        }

        program
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

            _ => self.syntactic_error(&format!("\"{}\" cannot start a command", self.current_token.spelling)),
        }
    }

    fn parse_identifier(&mut self) -> Identifier {
        if self.current_token.kind == TokenKind::Identifier {
            self.previous_token_position = self.current_token.position.clone();
            let spelling = self.current_token.spelling.clone();
            let identifier = Identifier {
                spelling,
                position: self.previous_token_position.clone(),
            };
            self.current_token = self.lexical_analyser.scan();

            return identifier
        } else {
            self.syntactic_error("identifier expected here")
        }
    }

    fn parse_actual_parameter_sequence(&mut self) -> ActualParameterSequence {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        if self.current_token.kind == TokenKind::RParen {
           self.finish(&mut position);
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Empty),
                position,
            }
        } else {
            self.parse_proper_actual_parameter_sequence()
        }
    }

    fn parse_proper_actual_parameter_sequence(&mut self) -> ActualParameterSequence {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let actual_parameter = self.parse_actual_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let actual_parameter_sequence = self.parse_proper_actual_parameter_sequence();
            self.finish(&mut position);
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Multiple {
                    parameter: actual_parameter,
                    rest: actual_parameter_sequence,
                }),
                position,
            }
        } else {
            self.finish(&mut position);
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Single(actual_parameter)),
                position,
            }
        }
    }

    fn parse_actual_parameter(&mut self) -> ActualParameter {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::Identifier | TokenKind::IntLiteral | TokenKind::CharLiteral
            | TokenKind::Operator | TokenKind::Let | TokenKind::If | TokenKind::LParen
            | TokenKind::LBracket | TokenKind::LCurly => {
                let expression = self.parse_expression();
                self.finish(&mut position);
                ActualParameter {
                    kind: ActualParameterKind::Const(expression),
                    position,
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let v_name = self.parse_v_name();
                self.finish(&mut position);
                ActualParameter {
                    kind: ActualParameterKind::Var(v_name),
                    position,
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.finish(&mut position);
                ActualParameter {
                    kind: ActualParameterKind::Proc(identifier),
                    position,
                }
            }

            TokenKind::Func => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.finish(&mut position);
                ActualParameter {
                    kind: ActualParameterKind::Func(identifier),
                    position,
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start an actual parameter", self.current_token.spelling)),
        }
    }

    fn parse_v_name(&mut self) -> VName {
        let identifier = self.parse_identifier();
        self.parse_rest_of_v_name(identifier)
    }

    fn parse_rest_of_v_name(&mut self, identifier: Identifier) -> VName {
        let mut position = identifier.position.clone();
        let mut v_name = VName {
            kind: Box::new(VNameKind::Simple(identifier)),
            position: position.clone(),
        };

        while self.current_token.kind == TokenKind::Dot || self.current_token.kind == TokenKind::LBracket {
            if self.current_token.kind == TokenKind::Dot {
                self.accept_it();
                v_name = VName {
                    kind: Box::new(VNameKind::Dot {
                        v_name,
                        field: self.parse_identifier()
                    }),
                    position: position.clone(),
                };
            } else {
                self.accept_it();
                let expression = self.parse_expression();
                self.accept(TokenKind::RBracket);
                self.finish(&mut position);
                v_name = VName {
                    kind: Box::new(VNameKind::Subscript {
                        v_name,
                        index: expression,
                    }),
                    position: position.clone(),
                }
            }
        }

        v_name
    }

    fn parse_expression(&mut self) -> Expression {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::Let => {
                self.accept_it();
                let declaration = self.parse_declaration();
                self.accept(TokenKind::In);
                let expression = self.parse_expression();
                self.finish(&mut position);

                Expression {
                    kind: Box::new(ExpressionKind::Let {
                        declaration,
                        body: expression,
                    }),
                    position,
                }
            }

            TokenKind::If => {
                self.accept_it();
                let cond_expression = self.parse_expression();
                self.accept(TokenKind::Then);
                let then_expression = self.parse_expression();
                self.accept(TokenKind::Else);
                let else_expression = self.parse_expression();
                self.finish(&mut position);

                Expression {
                    kind: Box::new(ExpressionKind::If {
                        condition: cond_expression,
                        then_branch: then_expression,
                        else_branch: else_expression,
                    }),
                    position,
                }
            }

            _ => {
                self.parse_secondary_expression()
            }
        }
    }

    fn parse_secondary_expression(&mut self) -> Expression {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let mut expression = self.parse_primary_expression();
        while self.current_token.kind == TokenKind::Operator {
            let operator = self.parse_operator();
            let second_expression = self.parse_primary_expression();

            expression = Expression {
                kind: Box::new(ExpressionKind::Binary {
                    left: expression,
                    operator,
                    right: second_expression,
                }),
                position: position.clone(),
            }
        }

        expression
    }

    fn parse_primary_expression(&mut self) -> Expression {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::IntLiteral => {
                let literal = self.parse_int_literal();
                self.finish(&mut position);
                Expression {
                    kind: Box::new(ExpressionKind::Integer(literal)),
                    position,
                }
            }

            TokenKind::CharLiteral => {
                let literal = self.parse_char_literal();
                self.finish(&mut position);
                Expression {
                    kind: Box::new(ExpressionKind::Character(literal)),
                    position,
                }
            }

            TokenKind::LBracket => {
                self.accept_it();
                let aggregate = self.parse_array_aggregate();
                self.accept(TokenKind::RBracket);
                self.finish(&mut position);
                Expression {
                    kind: Box::new(ExpressionKind::Array(aggregate)),
                    position,
                }
            }

            TokenKind::LCurly => {
                self.accept_it();
                let aggregate = self.parse_record_aggregate();
                self.accept(TokenKind::RCurly);
                self.finish(&mut position);
                Expression {
                    kind: Box::new(ExpressionKind::Record(aggregate)),
                    position,
                }
            }

            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);
                    self.finish(&mut position);
                    Expression {
                        kind: Box::new(ExpressionKind::Call { identifier, arguments }),
                        position,
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    self.finish(&mut position);
                    Expression {
                        kind: Box::new(ExpressionKind::VName(v_name)),
                        position,
                    }
                }
            }

            TokenKind::Operator => {
                let operator = self.parse_operator();
                let operand = self.parse_primary_expression();
                self.finish(&mut position);
                Expression {
                    kind: Box::new(ExpressionKind::Unary { operator, operand }),
                    position,
                }
            }

            TokenKind::LParen => {
                self.accept_it();
                let expression = self.parse_expression();
                self.accept(TokenKind::RParen);
                expression
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start an expression", self.current_token.spelling)),
        }
    }

    fn parse_int_literal(&mut self) -> IntegerLiteral {
        if self.current_token.kind == TokenKind::IntLiteral {
            self.previous_token_position = self.current_token.position.clone();
            let literal = IntegerLiteral {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position.clone(),
            };
            self.current_token = self.lexical_analyser.scan();

            literal
        } else {
            self.syntactic_error("integer literal expected here")
        }
    }

    fn parse_char_literal(&mut self) -> CharacterLiteral {
        if self.current_token.kind == TokenKind::CharLiteral {
            self.previous_token_position = self.current_token.position.clone();
            let literal = CharacterLiteral {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position.clone(),
            };
            self.current_token = self.lexical_analyser.scan();

            literal
        } else {
            self.syntactic_error("character literal expected here")
        }
    }

    fn parse_operator(&mut self) -> Operator {
        if self.current_token.kind == TokenKind::Operator {
            self.previous_token_position = self.current_token.position.clone();
            let operator = Operator {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position.clone(),
            };
            self.current_token = self.lexical_analyser.scan();

            operator
        } else {
            self.syntactic_error("operator expected here")
        }
    }

    fn parse_record_aggregate(&mut self) -> RecordAggregate {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Is);
        let expression = self.parse_expression();

        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_record_aggregate();
            self.finish(&mut position);
            RecordAggregate {
                kind: Box::new(RecordAggregateKind::Multiple { identifier, expression, rest }),
                position,
            }
        } else {
            self.finish(&mut position);
            RecordAggregate {
                kind: Box::new(RecordAggregateKind::Single { identifier, expression }),
                position,
            }
        }
    }

    fn parse_array_aggregate(&mut self) -> ArrayAggregate {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let expression = self.parse_expression();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_array_aggregate();
            self.finish(&mut position);
            ArrayAggregate {
                kind: Box::new(ArrayAggregateKind::Multiple { expression, rest }),
                position,
            }
        } else {
            self.finish(&mut position);
            ArrayAggregate {
                kind: Box::new(ArrayAggregateKind::Single(expression)),
                position,
            }
        }
    }

    fn parse_declaration(&mut self) -> Declaration {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let mut declaration = self.parse_single_declaration();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            let chained_declaration = self.parse_single_declaration();
            self.finish(&mut position);
            declaration = Declaration {
                kind: Box::new(DeclarationKind::Sequential {
                    first: declaration,
                    second: chained_declaration,
                }),
                position: position.clone(),
            }
        }

        declaration
    }

    fn parse_single_declaration(&mut self) -> Declaration {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::Const => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let expression = self.parse_expression();
                self.finish(&mut position);
                Declaration {
                    kind: Box::new(DeclarationKind::Const { identifier, expression }),
                    position,
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                self.finish(&mut position);
                Declaration {
                    kind: Box::new(DeclarationKind::Var { identifier, type_denoter }),
                    position,
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::LParen);
                let parameters = self.parse_formal_parameter_sequence();
                self.accept(TokenKind::RParen);
                self.accept(TokenKind::Is);
                let body = self.parse_single_command();
                self.finish(&mut position);
                Declaration {
                    kind: Box::new(DeclarationKind::Proc { identifier, parameters, body }),
                    position,
                }
            }

            TokenKind::Func => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::LParen);
                let parameters = self.parse_formal_parameter_sequence();
                self.accept(TokenKind::RParen);
                self.accept(TokenKind::Colon);
                let return_type = self.parse_type_denoter();
                self.accept(TokenKind::Is);
                let body = self.parse_expression();
                self.finish(&mut position);
                Declaration {
                    kind: Box::new(DeclarationKind::Func { identifier, parameters, return_type, body }),
                    position,
                }
            }

            TokenKind::Type => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let type_denoter = self.parse_type_denoter();
                self.finish(&mut position);
                Declaration {
                    kind: Box::new(DeclarationKind::Type { identifier, type_denoter }),
                    position,
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a declaration", self.current_token.spelling)),
        }
    }

    fn parse_formal_parameter_sequence(&mut self) -> FormalParameterSequence {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        if self.current_token.kind == TokenKind::RParen {
            self.finish(&mut position);
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Empty),
                position,
            }
        } else {
            self.parse_proper_formal_parameter_sequence()
        }
    }

    fn parse_proper_formal_parameter_sequence(&mut self) -> FormalParameterSequence {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let formal_parameter = self.parse_formal_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let formal_parameter_sequence = self.parse_proper_formal_parameter_sequence();
            self.finish(&mut position);
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Multiple {
                    parameter: formal_parameter,
                    rest: formal_parameter_sequence,
                }),
                position,
            }
        } else {
            self.finish(&mut position);
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Single(formal_parameter)),
                position,
            }
        }
    }

    fn parse_formal_parameter(&mut self) -> FormalParameter {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                self.finish(&mut position);
                FormalParameter {
                    kind: FormalParameterKind::Const { identifier, type_denoter },
                    position,
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                self.finish(&mut position);
                FormalParameter {
                    kind: FormalParameterKind::Var { identifier, type_denoter },
                    position,
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::LParen);
                let parameters = self.parse_formal_parameter_sequence();
                self.accept(TokenKind::RParen);
                self.finish(&mut position);
                FormalParameter {
                    kind: FormalParameterKind::Proc { identifier, parameters },
                    position,
                }
            }

            TokenKind::Func => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::LParen);
                let parameters = self.parse_formal_parameter_sequence();
                self.accept(TokenKind::RParen);
                self.accept(TokenKind::Colon);
                let return_type = self.parse_type_denoter();
                self.finish(&mut position);
                FormalParameter {
                    kind: FormalParameterKind::Func { identifier, parameters, return_type },
                    position,
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a formal parameter", self.current_token.spelling)),
        }
    }

    fn parse_type_denoter(&mut self) -> TypeDenoter {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                self.finish(&mut position);
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Simple(identifier)),
                    position,
                }
            }

            TokenKind::Array => {
                self.accept_it();
                let size = self.parse_int_literal();
                self.accept(TokenKind::Of);
                let element_type = self.parse_type_denoter();
                self.finish(&mut position);
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Array { size, element_type }),
                    position,
                }
            }

            TokenKind::Record => {
                self.accept_it();
                let fields = self.parse_field_type_denoter();
                self.accept(TokenKind::End);
                self.finish(&mut position);
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Record(fields)),
                    position,
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a type denoter", self.current_token.spelling)),
        }
    }

    fn parse_field_type_denoter(&mut self) -> FieldTypeDenoter {
        let mut position = SourcePosition {
            start: 0,
            finish: 0,
        };
        self.start(&mut position);

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Colon);
        let type_denoter = self.parse_type_denoter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_field_type_denoter();
            self.finish(&mut position);
            FieldTypeDenoter {
                kind: Box::new(FieldTypeDenoterKind::Multiple { identifier, type_denoter, rest }),
                position,
            }
        } else {
            self.finish(&mut position);
            FieldTypeDenoter {
                kind: Box::new(FieldTypeDenoterKind::Single { identifier, type_denoter }),
                position,
            }
        }
    }
}
