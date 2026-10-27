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
    /// Creates a new parser and scans the first token.
    ///
    /// <p> The first token is scanned immediately to avoid
    /// making current_token an optional type!
    /// This makes the implementations slightly different.
    pub fn new(mut scanner: Scanner<R>) -> Self {
        Self {
            current_token: scanner.scan(),
            lexical_analyser: scanner,
            previous_token_position: SourcePosition::default(),
        }
    }

    fn accept(&mut self, token_expected: TokenKind) {
        if self.current_token.kind == token_expected {
            self.previous_token_position = self.current_token.position;
            self.current_token = self.lexical_analyser.scan();
        } else {
            self.syntactic_error(&format!("\"{}\" expected here", token_expected.spelling()))
        }
    }

    fn accept_it(&mut self) {
        self.previous_token_position = self.current_token.position;
        self.current_token = self.lexical_analyser.scan();
    }

    fn start_pos(&self) -> i32 {
        self.current_token.position.start
    }

    fn finish_pos(&self) -> i32 {
        self.previous_token_position.finish
    }

    // Same output as the Java ErrorReporter, but aborts instead of throwing SyntaxError.
    fn syntactic_error(&self, message: &str) -> ! {
        let position = &self.current_token.position;
        panic!("ERROR: {} {}..{}", message, position.start, position.finish)
    }

    pub fn parse_program(&mut self) -> Program {
        // The first token was already scanned by `new`.
        self.previous_token_position = SourcePosition::default();
        let command = self.parse_command();
        let program = Program {
            command,
            position: self.previous_token_position,
        };
        if self.current_token.kind != TokenKind::Eof {
            self.syntactic_error(&format!("\"{}\" not expected after end of program", self.current_token.spelling));
        }

        program
    }

    fn parse_command(&mut self) -> Command {
        let start = self.start_pos();
        let mut finish = 0;

        let mut command = self.parse_single_command();
        let mut chained_commands = Vec::new();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            chained_commands.push(self.parse_single_command());
            finish = self.finish_pos();
        }

        // Java hands one SourcePosition object to every SequentialCommand of the chain and keeps
        // moving its finish, so they all end up with the finish of the last command.
        for chained_command in chained_commands {
            command = Command {
                kind: Box::from(CommandKind::Sequential {
                    first: command,
                    second: chained_command,
                }),
                position: SourcePosition::new(start, finish),
            }
        }

        command
    }

    fn parse_single_command(&mut self) -> Command {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);

                    Command {
                        kind: Box::new(CommandKind::Call { identifier, arguments }),
                        position: SourcePosition::new(start, self.finish_pos()),
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    self.accept(TokenKind::Becomes);
                    let expression = self.parse_expression();

                    Command {
                        kind: Box::new(CommandKind::Assign { v_name, expression }),
                        position: SourcePosition::new(start, self.finish_pos()),
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

                Command {
                    kind: Box::new(CommandKind::Let { declaration, body }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::If => {
                self.accept_it();
                let condition = self.parse_expression();
                self.accept(TokenKind::Then);
                let then_branch = self.parse_single_command();
                self.accept(TokenKind::Else);
                let else_branch = self.parse_single_command();

                Command {
                    kind: Box::new(CommandKind::If { condition, then_branch, else_branch }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::While => {
                self.accept_it();
                let condition = self.parse_expression();
                self.accept(TokenKind::Do);
                let body = self.parse_single_command();

                Command {
                    kind: Box::new(CommandKind::While { condition, body }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Semicolon | TokenKind::End | TokenKind::Else | TokenKind::In | TokenKind::Eof => {
                Command {
                    kind: Box::new(CommandKind::Empty),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a command", self.current_token.spelling)),
        }
    }

    fn parse_identifier(&mut self) -> Identifier {
        if self.current_token.kind == TokenKind::Identifier {
            self.previous_token_position = self.current_token.position;
            let spelling = self.current_token.spelling.clone();
            let identifier = Identifier {
                spelling,
                position: self.previous_token_position,
            };
            self.current_token = self.lexical_analyser.scan();

            return identifier
        } else {
            self.syntactic_error("identifier expected here")
        }
    }

    fn parse_actual_parameter_sequence(&mut self) -> ActualParameterSequence {
        let start = self.start_pos();

        if self.current_token.kind == TokenKind::RParen {
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Empty),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            self.parse_proper_actual_parameter_sequence()
        }
    }

    fn parse_proper_actual_parameter_sequence(&mut self) -> ActualParameterSequence {
        let start = self.start_pos();

        let actual_parameter = self.parse_actual_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let actual_parameter_sequence = self.parse_proper_actual_parameter_sequence();
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Multiple {
                    parameter: actual_parameter,
                    rest: actual_parameter_sequence,
                }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            ActualParameterSequence {
                kind: Box::new(ActualParameterSequenceKind::Single(actual_parameter)),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        }
    }

    fn parse_actual_parameter(&mut self) -> ActualParameter {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier | TokenKind::IntLiteral | TokenKind::CharLiteral
            | TokenKind::Operator | TokenKind::Let | TokenKind::If | TokenKind::LParen
            | TokenKind::LBracket | TokenKind::LCurly => {
                let expression = self.parse_expression();
                ActualParameter {
                    kind: ActualParameterKind::Const(expression),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let v_name = self.parse_v_name();
                ActualParameter {
                    kind: ActualParameterKind::Var(v_name),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                ActualParameter {
                    kind: ActualParameterKind::Proc(identifier),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Func => {
                self.accept_it();
                let identifier = self.parse_identifier();
                ActualParameter {
                    kind: ActualParameterKind::Func(identifier),
                    position: SourcePosition::new(start, self.finish_pos()),
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
        enum Selector {
            Field(Identifier),
            Index(Expression),
        }

        let mut position = identifier.position;
        let mut selectors = Vec::new();
        while self.current_token.kind == TokenKind::Dot || self.current_token.kind == TokenKind::LBracket {
            if self.current_token.kind == TokenKind::Dot {
                self.accept_it();
                selectors.push(Selector::Field(self.parse_identifier()));
            } else {
                self.accept_it();
                let expression = self.parse_expression();
                self.accept(TokenKind::RBracket);
                position.finish = self.finish_pos();
                selectors.push(Selector::Index(expression));
            }
        }

        // Java uses the identifier's own SourcePosition object for every vname of the chain and
        // finish() moves it on each subscript, so the identifier and all the vnames end up with
        // the finish of the last `]`.
        let identifier = Identifier {
            position,
            ..identifier
        };
        let mut v_name = VName {
            kind: Box::new(VNameKind::Simple(identifier)),
            position,
        };
        for selector in selectors {
            let kind = match selector {
                Selector::Field(field) => VNameKind::Dot { v_name, field },
                Selector::Index(index) => VNameKind::Subscript { v_name, index },
            };
            v_name = VName {
                kind: Box::new(kind),
                position,
            };
        }

        v_name
    }

    fn parse_expression(&mut self) -> Expression {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Let => {
                self.accept_it();
                let declaration = self.parse_declaration();
                self.accept(TokenKind::In);
                let expression = self.parse_expression();

                Expression {
                    kind: Box::new(ExpressionKind::Let {
                        declaration,
                        body: expression,
                    }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::If => {
                self.accept_it();
                let cond_expression = self.parse_expression();
                self.accept(TokenKind::Then);
                let then_expression = self.parse_expression();
                self.accept(TokenKind::Else);
                let else_expression = self.parse_expression();

                Expression {
                    kind: Box::new(ExpressionKind::If {
                        condition: cond_expression,
                        then_branch: then_expression,
                        else_branch: else_expression,
                    }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            _ => {
                self.parse_secondary_expression()
            }
        }
    }

    fn parse_secondary_expression(&mut self) -> Expression {
        let position = SourcePosition::new(self.start_pos(), 0);

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
                position,
            }
        }

        expression
    }

    fn parse_primary_expression(&mut self) -> Expression {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::IntLiteral => {
                let literal = self.parse_int_literal();
                Expression {
                    kind: Box::new(ExpressionKind::Integer(literal)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::CharLiteral => {
                let literal = self.parse_char_literal();
                Expression {
                    kind: Box::new(ExpressionKind::Character(literal)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::LBracket => {
                self.accept_it();
                let aggregate = self.parse_array_aggregate();
                self.accept(TokenKind::RBracket);
                Expression {
                    kind: Box::new(ExpressionKind::Array(aggregate)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::LCurly => {
                self.accept_it();
                let aggregate = self.parse_record_aggregate();
                self.accept(TokenKind::RCurly);
                Expression {
                    kind: Box::new(ExpressionKind::Record(aggregate)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);
                    Expression {
                        kind: Box::new(ExpressionKind::Call { identifier, arguments }),
                        position: SourcePosition::new(start, self.finish_pos()),
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    Expression {
                        kind: Box::new(ExpressionKind::VName(v_name)),
                        position: SourcePosition::new(start, self.finish_pos()),
                    }
                }
            }

            TokenKind::Operator => {
                let operator = self.parse_operator();
                let operand = self.parse_primary_expression();
                Expression {
                    kind: Box::new(ExpressionKind::Unary { operator, operand }),
                    position: SourcePosition::new(start, self.finish_pos()),
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
            self.previous_token_position = self.current_token.position;
            let literal = IntegerLiteral {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position,
            };
            self.current_token = self.lexical_analyser.scan();

            literal
        } else {
            self.syntactic_error("integer literal expected here")
        }
    }

    fn parse_char_literal(&mut self) -> CharacterLiteral {
        if self.current_token.kind == TokenKind::CharLiteral {
            self.previous_token_position = self.current_token.position;
            let literal = CharacterLiteral {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position,
            };
            self.current_token = self.lexical_analyser.scan();

            literal
        } else {
            self.syntactic_error("character literal expected here")
        }
    }

    fn parse_operator(&mut self) -> Operator {
        if self.current_token.kind == TokenKind::Operator {
            self.previous_token_position = self.current_token.position;
            let operator = Operator {
                spelling: self.current_token.spelling.clone(),
                position: self.previous_token_position,
            };
            self.current_token = self.lexical_analyser.scan();

            operator
        } else {
            self.syntactic_error("operator expected here")
        }
    }

    fn parse_record_aggregate(&mut self) -> RecordAggregate {
        let start = self.start_pos();

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Is);
        let expression = self.parse_expression();

        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_record_aggregate();
            RecordAggregate {
                kind: Box::new(RecordAggregateKind::Multiple { identifier, expression, rest }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            RecordAggregate {
                kind: Box::new(RecordAggregateKind::Single { identifier, expression }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        }
    }

    fn parse_array_aggregate(&mut self) -> ArrayAggregate {
        let start = self.start_pos();

        let expression = self.parse_expression();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_array_aggregate();
            ArrayAggregate {
                kind: Box::new(ArrayAggregateKind::Multiple { expression, rest }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            ArrayAggregate {
                kind: Box::new(ArrayAggregateKind::Single(expression)),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        }
    }

    fn parse_declaration(&mut self) -> Declaration {
        let start = self.start_pos();
        let mut finish = 0;

        let mut declaration = self.parse_single_declaration();
        let mut chained_declarations = Vec::new();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            chained_declarations.push(self.parse_single_declaration());
            finish = self.finish_pos();
        }

        // Same position sharing as in parse_command.
        for chained_declaration in chained_declarations {
            declaration = Declaration {
                kind: Box::new(DeclarationKind::Sequential {
                    first: declaration,
                    second: chained_declaration,
                }),
                position: SourcePosition::new(start, finish),
            }
        }

        declaration
    }

    fn parse_single_declaration(&mut self) -> Declaration {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Const => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let expression = self.parse_expression();
                Declaration {
                    kind: Box::new(DeclarationKind::Const { identifier, expression }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                Declaration {
                    kind: Box::new(DeclarationKind::Var { identifier, type_denoter }),
                    position: SourcePosition::new(start, self.finish_pos()),
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
                Declaration {
                    kind: Box::new(DeclarationKind::Proc { identifier, parameters, body }),
                    position: SourcePosition::new(start, self.finish_pos()),
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
                Declaration {
                    kind: Box::new(DeclarationKind::Func { identifier, parameters, return_type, body }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Type => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let type_denoter = self.parse_type_denoter();
                Declaration {
                    kind: Box::new(DeclarationKind::Type { identifier, type_denoter }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a declaration", self.current_token.spelling)),
        }
    }

    fn parse_formal_parameter_sequence(&mut self) -> FormalParameterSequence {
        let start = self.start_pos();

        if self.current_token.kind == TokenKind::RParen {
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Empty),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            self.parse_proper_formal_parameter_sequence()
        }
    }

    fn parse_proper_formal_parameter_sequence(&mut self) -> FormalParameterSequence {
        let start = self.start_pos();

        let formal_parameter = self.parse_formal_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let formal_parameter_sequence = self.parse_proper_formal_parameter_sequence();
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Multiple {
                    parameter: formal_parameter,
                    rest: formal_parameter_sequence,
                }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            FormalParameterSequence {
                kind: Box::new(FormalParameterSequenceKind::Single(formal_parameter)),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        }
    }

    fn parse_formal_parameter(&mut self) -> FormalParameter {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                FormalParameter {
                    kind: FormalParameterKind::Const { identifier, type_denoter },
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                FormalParameter {
                    kind: FormalParameterKind::Var { identifier, type_denoter },
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::LParen);
                let parameters = self.parse_formal_parameter_sequence();
                self.accept(TokenKind::RParen);
                FormalParameter {
                    kind: FormalParameterKind::Proc { identifier, parameters },
                    position: SourcePosition::new(start, self.finish_pos()),
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
                FormalParameter {
                    kind: FormalParameterKind::Func { identifier, parameters, return_type },
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a formal parameter", self.current_token.spelling)),
        }
    }

    fn parse_type_denoter(&mut self) -> TypeDenoter {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Simple(identifier)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Array => {
                self.accept_it();
                let size = self.parse_int_literal();
                self.accept(TokenKind::Of);
                let element_type = self.parse_type_denoter();
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Array { size, element_type }),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            TokenKind::Record => {
                self.accept_it();
                let fields = self.parse_field_type_denoter();
                self.accept(TokenKind::End);
                TypeDenoter {
                    kind: Box::new(TypeDenoterKind::Record(fields)),
                    position: SourcePosition::new(start, self.finish_pos()),
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a type denoter", self.current_token.spelling)),
        }
    }

    fn parse_field_type_denoter(&mut self) -> FieldTypeDenoter {
        let start = self.start_pos();

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Colon);
        let type_denoter = self.parse_type_denoter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_field_type_denoter();
            FieldTypeDenoter {
                kind: Box::new(FieldTypeDenoterKind::Multiple { identifier, type_denoter, rest }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        } else {
            FieldTypeDenoter {
                kind: Box::new(FieldTypeDenoterKind::Single { identifier, type_denoter }),
                position: SourcePosition::new(start, self.finish_pos()),
            }
        }
    }
}
