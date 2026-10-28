use crate::abstract_syntax_trees::ast::{ActualParameter, ActualParameterKind, ActualParameterSequence, ActualParameterSequenceKind, ArrayAggregate, ArrayAggregateKind, CharacterLiteral, Command, CommandKind, Declaration, DeclarationKind, Expression, ExpressionKind, FieldTypeDenoter, FieldTypeDenoterKind, FormalParameter, FormalParameterKind, FormalParameterSequence, FormalParameterSequenceKind, Identifier, IntegerLiteral, Operator, Program, RecordAggregate, RecordAggregateKind, TypeDenoter, TypeDenoterKind, VName, VNameKind};
use bumpalo::Bump;

use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::{latin1_to_string, Token};
use crate::syntactic_analyzer::token_kind::TokenKind;
use crate::syntactic_analyzer::token_stream::TokenStream;

/// One `.field` or `[index]` after the identifier of a v_name.
enum Selector<'a> {
    Field(Identifier<'a>),
    Index(Expression<'a>),
}

pub struct Parser<'a> {
    current_token: Token<'a>,
    token_stream: TokenStream<'a>,
    previous_token_position: SourcePosition,
    arena: &'a Bump,
    command_stack: Vec<Command<'a>>,
    declaration_stack: Vec<Declaration<'a>>,
    selector_stack: Vec<Selector<'a>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser and scans the first token.
    ///
    /// <p> The first token is scanned immediately to avoid
    /// making current_token an optional type!
    /// This makes the implementations slightly different.
    /// Every AST node is allocated in `arena`, so the tree lives as long as the arena does.
    pub fn new(mut token_stream: TokenStream<'a>, arena: &'a Bump) -> Self {
        Self {
            current_token: token_stream.next(),
            token_stream,
            previous_token_position: SourcePosition::default(),
            arena,
            command_stack: vec![],
            declaration_stack: vec![],
            selector_stack: vec![],
        }
    }

    fn accept(&mut self, token_expected: TokenKind) {
        if self.current_token.kind == token_expected {
            self.accept_it();
        } else {
            self.syntactic_error(&format!("\"{}\" expected here", token_expected.spelling()))
        }
    }

    fn accept_it(&mut self) -> Token<'a> {
        let next = self.token_stream.next();
        let token = std::mem::replace(&mut self.current_token, next);
        self.previous_token_position = token.position;
        token
    }

    fn start_pos(&self) -> u32 {
        self.current_token.position.start
    }

    fn finish_pos(&self) -> u32 {
        self.previous_token_position.finish
    }

    // Same output as the Java ErrorReporter, but aborts instead of throwing SyntaxError.
    fn syntactic_error(&self, message: &str) -> ! {
        let position = &self.current_token.position;
        panic!("ERROR: {} {}..{}", message, position.start, position.finish)
    }

    pub fn parse_program(&mut self) -> Program<'a> {
        // The first token was already scanned by `new`.
        self.previous_token_position = SourcePosition::default();
        let command = self.parse_command();
        let program = Program {
            command,
            position: self.previous_token_position,
        };
        if self.current_token.kind != TokenKind::Eof {
            self.syntactic_error(&format!("\"{}\" not expected after end of program", latin1_to_string(self.current_token.spelling)));
        }

        program
    }

    fn parse_command(&mut self) -> Command<'a> {
        let start = self.start_pos();

        let mut command = self.parse_single_command();
        // The chained commands go on the shared stack above `base`; nested calls push above us
        // and drain back down before they return, so our part stays intact.
        let base = self.command_stack.len();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            let chained_command = self.parse_single_command();
            self.command_stack.push(chained_command);
        }

        // Java hands one SourcePosition object to every SequentialCommand of the chain and keeps
        // moving its finish, so they all end up with the finish of the last command.
        let finish = self.finish_pos();
        let arena = self.arena;
        for chained_command in self.command_stack.drain(base..) {
            command = Command {
                kind: arena.alloc(CommandKind::Sequential {
                    first: command,
                    second: chained_command,
                }),
                position: SourcePosition { start, finish },
            }
        }

        command
    }

    fn parse_single_command(&mut self) -> Command<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);

                    Command {
                        kind: self.arena.alloc(CommandKind::Call { identifier, arguments }),
                        position: SourcePosition { start, finish: self.finish_pos() },
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    self.accept(TokenKind::Becomes);
                    let expression = self.parse_expression();

                    Command {
                        kind: self.arena.alloc(CommandKind::Assign { v_name, expression }),
                        position: SourcePosition { start, finish: self.finish_pos() },
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
                    kind: self.arena.alloc(CommandKind::Let { declaration, body }),
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    kind: self.arena.alloc(CommandKind::If { condition, then_branch, else_branch }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::While => {
                self.accept_it();
                let condition = self.parse_expression();
                self.accept(TokenKind::Do);
                let body = self.parse_single_command();

                Command {
                    kind: self.arena.alloc(CommandKind::While { condition, body }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Semicolon | TokenKind::End | TokenKind::Else | TokenKind::In | TokenKind::Eof => {
                Command {
                    kind: self.arena.alloc(CommandKind::Empty),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a command", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_identifier(&mut self) -> Identifier<'a> {
        if self.current_token.kind == TokenKind::Identifier {
            let token = self.accept_it();
            Identifier { spelling: token.spelling, position: token.position }
        } else {
            self.syntactic_error("identifier expected here")
        }
    }

    fn parse_actual_parameter_sequence(&mut self) -> ActualParameterSequence<'a> {
        let start = self.start_pos();

        if self.current_token.kind == TokenKind::RParen {
            ActualParameterSequence {
                kind: self.arena.alloc(ActualParameterSequenceKind::Empty),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            self.parse_proper_actual_parameter_sequence()
        }
    }

    fn parse_proper_actual_parameter_sequence(&mut self) -> ActualParameterSequence<'a> {
        let start = self.start_pos();

        let actual_parameter = self.parse_actual_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let actual_parameter_sequence = self.parse_proper_actual_parameter_sequence();
            ActualParameterSequence {
                kind: self.arena.alloc(ActualParameterSequenceKind::Multiple {
                    parameter: actual_parameter,
                    rest: actual_parameter_sequence,
                }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            ActualParameterSequence {
                kind: self.arena.alloc(ActualParameterSequenceKind::Single(actual_parameter)),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        }
    }

    fn parse_actual_parameter(&mut self) -> ActualParameter<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier | TokenKind::IntLiteral | TokenKind::CharLiteral
            | TokenKind::Operator | TokenKind::Let | TokenKind::If | TokenKind::LParen
            | TokenKind::LBracket | TokenKind::LCurly => {
                let expression = self.parse_expression();
                ActualParameter {
                    kind: ActualParameterKind::Const(expression),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let v_name = self.parse_v_name();
                ActualParameter {
                    kind: ActualParameterKind::Var(v_name),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Proc => {
                self.accept_it();
                let identifier = self.parse_identifier();
                ActualParameter {
                    kind: ActualParameterKind::Proc(identifier),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Func => {
                self.accept_it();
                let identifier = self.parse_identifier();
                ActualParameter {
                    kind: ActualParameterKind::Func(identifier),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start an actual parameter", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_v_name(&mut self) -> VName<'a> {
        let identifier = self.parse_identifier();
        self.parse_rest_of_v_name(identifier)
    }

    fn parse_rest_of_v_name(&mut self, identifier: Identifier<'a>) -> VName<'a> {
        let mut position = identifier.position;
        // Same shared-stack scheme as in parse_command.
        let base = self.selector_stack.len();
        while self.current_token.kind == TokenKind::Dot || self.current_token.kind == TokenKind::LBracket {
            if self.current_token.kind == TokenKind::Dot {
                self.accept_it();
                let field = self.parse_identifier();
                self.selector_stack.push(Selector::Field(field));
            } else {
                self.accept_it();
                let expression = self.parse_expression();
                self.accept(TokenKind::RBracket);
                position.finish = self.finish_pos();
                self.selector_stack.push(Selector::Index(expression));
            }
        }

        // Java uses the identifier's own SourcePosition object for every v_name of the chain and
        // finish() moves it on each subscript, so the identifier and all the v_names end up with
        // the finish of the last `]`.
        let identifier = Identifier {
            position,
            ..identifier
        };
        let mut v_name = VName {
            kind: self.arena.alloc(VNameKind::Simple(identifier)),
            position,
        };
        let arena = self.arena;
        for selector in self.selector_stack.drain(base..) {
            let kind = match selector {
                Selector::Field(field) => VNameKind::Dot { v_name, field },
                Selector::Index(index) => VNameKind::Subscript { v_name, index },
            };
            v_name = VName {
                kind: arena.alloc(kind),
                position,
            };
        }

        v_name
    }

    fn parse_expression(&mut self) -> Expression<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Let => {
                self.accept_it();
                let declaration = self.parse_declaration();
                self.accept(TokenKind::In);
                let expression = self.parse_expression();

                Expression {
                    kind: self.arena.alloc(ExpressionKind::Let {
                        declaration,
                        body: expression,
                    }),
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    kind: self.arena.alloc(ExpressionKind::If {
                        condition: cond_expression,
                        then_branch: then_expression,
                        else_branch: else_expression,
                    }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => {
                self.parse_secondary_expression()
            }
        }
    }

    fn parse_secondary_expression(&mut self) -> Expression<'a> {
        let position = SourcePosition { start: self.start_pos(), finish: 0 };

        let mut expression = self.parse_primary_expression();
        while self.current_token.kind == TokenKind::Operator {
            let operator = self.parse_operator();
            let second_expression = self.parse_primary_expression();

            expression = Expression {
                kind: self.arena.alloc(ExpressionKind::Binary {
                    left: expression,
                    operator,
                    right: second_expression,
                }),
                position,
            }
        }

        expression
    }

    fn parse_primary_expression(&mut self) -> Expression<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::IntLiteral => {
                let literal = self.parse_int_literal();
                Expression {
                    kind: self.arena.alloc(ExpressionKind::Integer(literal)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::CharLiteral => {
                let literal = self.parse_char_literal();
                Expression {
                    kind: self.arena.alloc(ExpressionKind::Character(literal)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::LBracket => {
                self.accept_it();
                let aggregate = self.parse_array_aggregate();
                self.accept(TokenKind::RBracket);
                Expression {
                    kind: self.arena.alloc(ExpressionKind::Array(aggregate)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::LCurly => {
                self.accept_it();
                let aggregate = self.parse_record_aggregate();
                self.accept(TokenKind::RCurly);
                Expression {
                    kind: self.arena.alloc(ExpressionKind::Record(aggregate)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                if self.current_token.kind == TokenKind::LParen {
                    self.accept_it();
                    let arguments = self.parse_actual_parameter_sequence();
                    self.accept(TokenKind::RParen);
                    Expression {
                        kind: self.arena.alloc(ExpressionKind::Call { identifier, arguments }),
                        position: SourcePosition { start, finish: self.finish_pos() },
                    }
                } else {
                    let v_name = self.parse_rest_of_v_name(identifier);
                    Expression {
                        kind: self.arena.alloc(ExpressionKind::VName(v_name)),
                        position: SourcePosition { start, finish: self.finish_pos() },
                    }
                }
            }

            TokenKind::Operator => {
                let operator = self.parse_operator();
                let operand = self.parse_primary_expression();
                Expression {
                    kind: self.arena.alloc(ExpressionKind::Unary { operator, operand }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::LParen => {
                self.accept_it();
                let expression = self.parse_expression();
                self.accept(TokenKind::RParen);
                expression
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start an expression", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_int_literal(&mut self) -> IntegerLiteral<'a> {
        if self.current_token.kind == TokenKind::IntLiteral {
            let token = self.accept_it();
            IntegerLiteral { spelling: token.spelling, position: token.position }
        } else {
            self.syntactic_error("integer literal expected here")
        }
    }

    fn parse_char_literal(&mut self) -> CharacterLiteral<'a> {
        if self.current_token.kind == TokenKind::CharLiteral {
            let token = self.accept_it();
            CharacterLiteral { spelling: token.spelling, position: token.position }
        } else {
            self.syntactic_error("character literal expected here")
        }
    }

    fn parse_operator(&mut self) -> Operator<'a> {
        if self.current_token.kind == TokenKind::Operator {
            let token = self.accept_it();
            Operator { spelling: token.spelling, position: token.position }
        } else {
            self.syntactic_error("operator expected here")
        }
    }

    fn parse_record_aggregate(&mut self) -> RecordAggregate<'a> {
        let start = self.start_pos();

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Is);
        let expression = self.parse_expression();

        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_record_aggregate();
            RecordAggregate {
                kind: self.arena.alloc(RecordAggregateKind::Multiple { identifier, expression, rest }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            RecordAggregate {
                kind: self.arena.alloc(RecordAggregateKind::Single { identifier, expression }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        }
    }

    fn parse_array_aggregate(&mut self) -> ArrayAggregate<'a> {
        let start = self.start_pos();

        let expression = self.parse_expression();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_array_aggregate();
            ArrayAggregate {
                kind: self.arena.alloc(ArrayAggregateKind::Multiple { expression, rest }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            ArrayAggregate {
                kind: self.arena.alloc(ArrayAggregateKind::Single(expression)),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        }
    }

    fn parse_declaration(&mut self) -> Declaration<'a> {
        let start = self.start_pos();

        let mut declaration = self.parse_single_declaration();
        // Same shared-stack scheme as in parse_command.
        let base = self.declaration_stack.len();
        while self.current_token.kind == TokenKind::Semicolon {
            self.accept_it();
            let chained_declaration = self.parse_single_declaration();
            self.declaration_stack.push(chained_declaration);
        }

        // Same position sharing as in parse_command.
        let finish = self.finish_pos();
        let arena = self.arena;
        for chained_declaration in self.declaration_stack.drain(base..) {
            declaration = Declaration {
                kind: arena.alloc(DeclarationKind::Sequential {
                    first: declaration,
                    second: chained_declaration,
                }),
                position: SourcePosition { start, finish },
            }
        }

        declaration
    }

    fn parse_single_declaration(&mut self) -> Declaration<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Const => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let expression = self.parse_expression();
                Declaration {
                    kind: self.arena.alloc(DeclarationKind::Const { identifier, expression }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                Declaration {
                    kind: self.arena.alloc(DeclarationKind::Var { identifier, type_denoter }),
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    kind: self.arena.alloc(DeclarationKind::Proc { identifier, parameters, body }),
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    kind: self.arena.alloc(DeclarationKind::Func { identifier, parameters, return_type, body }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Type => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Is);
                let type_denoter = self.parse_type_denoter();
                Declaration {
                    kind: self.arena.alloc(DeclarationKind::Type { identifier, type_denoter }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a declaration", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_formal_parameter_sequence(&mut self) -> FormalParameterSequence<'a> {
        let start = self.start_pos();

        if self.current_token.kind == TokenKind::RParen {
            FormalParameterSequence {
                kind: self.arena.alloc(FormalParameterSequenceKind::Empty),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            self.parse_proper_formal_parameter_sequence()
        }
    }

    fn parse_proper_formal_parameter_sequence(&mut self) -> FormalParameterSequence<'a> {
        let start = self.start_pos();

        let formal_parameter = self.parse_formal_parameter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let formal_parameter_sequence = self.parse_proper_formal_parameter_sequence();
            FormalParameterSequence {
                kind: self.arena.alloc(FormalParameterSequenceKind::Multiple {
                    parameter: formal_parameter,
                    rest: formal_parameter_sequence,
                }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            FormalParameterSequence {
                kind: self.arena.alloc(FormalParameterSequenceKind::Single(formal_parameter)),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        }
    }

    fn parse_formal_parameter(&mut self) -> FormalParameter<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                FormalParameter {
                    kind: FormalParameterKind::Const { identifier, type_denoter },
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Var => {
                self.accept_it();
                let identifier = self.parse_identifier();
                self.accept(TokenKind::Colon);
                let type_denoter = self.parse_type_denoter();
                FormalParameter {
                    kind: FormalParameterKind::Var { identifier, type_denoter },
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    position: SourcePosition { start, finish: self.finish_pos() },
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
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a formal parameter", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_type_denoter(&mut self) -> TypeDenoter<'a> {
        let start = self.start_pos();

        match self.current_token.kind {
            TokenKind::Identifier => {
                let identifier = self.parse_identifier();
                TypeDenoter {
                    kind: self.arena.alloc(TypeDenoterKind::Simple(identifier)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Array => {
                self.accept_it();
                let size = self.parse_int_literal();
                self.accept(TokenKind::Of);
                let element_type = self.parse_type_denoter();
                TypeDenoter {
                    kind: self.arena.alloc(TypeDenoterKind::Array { size, element_type }),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            TokenKind::Record => {
                self.accept_it();
                let fields = self.parse_field_type_denoter();
                self.accept(TokenKind::End);
                TypeDenoter {
                    kind: self.arena.alloc(TypeDenoterKind::Record(fields)),
                    position: SourcePosition { start, finish: self.finish_pos() },
                }
            }

            _ => self.syntactic_error(&format!("\"{}\" cannot start a type denoter", latin1_to_string(self.current_token.spelling))),
        }
    }

    fn parse_field_type_denoter(&mut self) -> FieldTypeDenoter<'a> {
        let start = self.start_pos();

        let identifier = self.parse_identifier();
        self.accept(TokenKind::Colon);
        let type_denoter = self.parse_type_denoter();
        if self.current_token.kind == TokenKind::Comma {
            self.accept_it();
            let rest = self.parse_field_type_denoter();
            FieldTypeDenoter {
                kind: self.arena.alloc(FieldTypeDenoterKind::Multiple { identifier, type_denoter, rest }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        } else {
            FieldTypeDenoter {
                kind: self.arena.alloc(FieldTypeDenoterKind::Single { identifier, type_denoter }),
                position: SourcePosition { start, finish: self.finish_pos() },
            }
        }
    }
}
