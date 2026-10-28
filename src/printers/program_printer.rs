use std::fmt::Write;

use crate::abstract_syntax_trees::ast::{ActualParameter, ActualParameterKind, ActualParameterSequence, ActualParameterSequenceKind, ArrayAggregate, ArrayAggregateKind, CharacterLiteral, Command, CommandKind, Declaration, DeclarationKind, Expression, ExpressionKind, FieldTypeDenoter, FieldTypeDenoterKind, FormalParameter, FormalParameterKind, FormalParameterSequence, FormalParameterSequenceKind, Identifier, IntegerLiteral, Operator, Program, RecordAggregate, RecordAggregateKind, TypeDenoter, TypeDenoterKind, VName, VNameKind};
use crate::syntactic_analyzer::source_position::SourcePosition;
use crate::syntactic_analyzer::token::latin1_to_string;

// Same output as ProgramPrinter.java in the Java reference: one node per line, children indented
// by two spaces, nodes named after their Java class so the two outputs can be diffed directly.
//   <ClassName> <start>..<finish>
//   <ClassName> "<spelling>" <start>..<finish>   (terminals)

pub fn print(program: &Program) -> String {
    let mut printer = Printer { out: String::new(), depth: 0 };
    printer.program(program);
    printer.out
}

struct Printer {
    out: String,
    depth: usize,
}

impl Printer {
    fn line(&mut self, name: &str, position: &SourcePosition) {
        writeln!(self.out, "{}{} {}..{}", "  ".repeat(self.depth), name, position.start, position.finish).unwrap();
    }

    fn terminal(&mut self, name: &str, spelling: &[u8], position: &SourcePosition) {
        writeln!(self.out, "{}{} \"{}\" {}..{}", "  ".repeat(self.depth), name, latin1_to_string(spelling), position.start, position.finish).unwrap();
    }

    // Prints the node's line, then runs `children` one level deeper.
    fn node(&mut self, name: &str, position: &SourcePosition, children: impl FnOnce(&mut Self)) {
        self.line(name, position);
        self.depth += 1;
        children(self);
        self.depth -= 1;
    }

    fn program(&mut self, program: &Program) {
        self.node("Program", &program.position, |p| p.command(&program.command));
    }

    fn command(&mut self, command: &Command) {
        let position = &command.position;
        match &*command.kind {
            CommandKind::Assign { v_name, expression } => self.node("AssignCommand", position, |p| {
                p.v_name(v_name);
                p.expression(expression);
            }),
            CommandKind::Call { identifier, arguments } => self.node("CallCommand", position, |p| {
                p.identifier(identifier);
                p.actual_parameter_sequence(arguments);
            }),
            CommandKind::Empty => self.line("EmptyCommand", position),
            CommandKind::If { condition, then_branch, else_branch } => self.node("IfCommand", position, |p| {
                p.expression(condition);
                p.command(then_branch);
                p.command(else_branch);
            }),
            CommandKind::Let { declaration, body } => self.node("LetCommand", position, |p| {
                p.declaration(declaration);
                p.command(body);
            }),
            CommandKind::Sequential { first, second } => self.node("SequentialCommand", position, |p| {
                p.command(first);
                p.command(second);
            }),
            CommandKind::While { condition, body } => self.node("WhileCommand", position, |p| {
                p.expression(condition);
                p.command(body);
            }),
        }
    }

    fn expression(&mut self, expression: &Expression) {
        let position = &expression.position;
        match &*expression.kind {
            ExpressionKind::Array(aggregate) => self.node("ArrayExpression", position, |p| p.array_aggregate(aggregate)),
            ExpressionKind::Binary { left, operator, right } => self.node("BinaryExpression", position, |p| {
                p.expression(left);
                p.operator(operator);
                p.expression(right);
            }),
            ExpressionKind::Call { identifier, arguments } => self.node("CallExpression", position, |p| {
                p.identifier(identifier);
                p.actual_parameter_sequence(arguments);
            }),
            ExpressionKind::Character(literal) => self.node("CharacterExpression", position, |p| p.character_literal(literal)),
            ExpressionKind::Empty => self.line("EmptyExpression", position),
            ExpressionKind::If { condition, then_branch, else_branch } => self.node("IfExpression", position, |p| {
                p.expression(condition);
                p.expression(then_branch);
                p.expression(else_branch);
            }),
            ExpressionKind::Integer(literal) => self.node("IntegerExpression", position, |p| p.integer_literal(literal)),
            ExpressionKind::Let { declaration, body } => self.node("LetExpression", position, |p| {
                p.declaration(declaration);
                p.expression(body);
            }),
            ExpressionKind::Record(aggregate) => self.node("RecordExpression", position, |p| p.record_aggregate(aggregate)),
            ExpressionKind::Unary { operator, operand } => self.node("UnaryExpression", position, |p| {
                p.operator(operator);
                p.expression(operand);
            }),
            ExpressionKind::VName(v_name) => self.node("VnameExpression", position, |p| p.v_name(v_name)),
        }
    }

    fn array_aggregate(&mut self, aggregate: &ArrayAggregate) {
        let position = &aggregate.position;
        match &*aggregate.kind {
            ArrayAggregateKind::Single(expression) => self.node("SingleArrayAggregate", position, |p| p.expression(expression)),
            ArrayAggregateKind::Multiple { expression, rest } => self.node("MultipleArrayAggregate", position, |p| {
                p.expression(expression);
                p.array_aggregate(rest);
            }),
        }
    }

    fn record_aggregate(&mut self, aggregate: &RecordAggregate) {
        let position = &aggregate.position;
        match &*aggregate.kind {
            RecordAggregateKind::Single { identifier, expression } => self.node("SingleRecordAggregate", position, |p| {
                p.identifier(identifier);
                p.expression(expression);
            }),
            RecordAggregateKind::Multiple { identifier, expression, rest } => self.node("MultipleRecordAggregate", position, |p| {
                p.identifier(identifier);
                p.expression(expression);
                p.record_aggregate(rest);
            }),
        }
    }

    fn v_name(&mut self, v_name: &VName) {
        let position = &v_name.position;
        match &*v_name.kind {
            VNameKind::Simple(identifier) => self.node("SimpleVname", position, |p| p.identifier(identifier)),
            VNameKind::Dot { v_name, field } => self.node("DotVname", position, |p| {
                p.v_name(v_name);
                p.identifier(field);
            }),
            VNameKind::Subscript { v_name, index } => self.node("SubscriptVname", position, |p| {
                p.v_name(v_name);
                p.expression(index);
            }),
        }
    }

    fn declaration(&mut self, declaration: &Declaration) {
        let position = &declaration.position;
        match &*declaration.kind {
            DeclarationKind::Const { identifier, expression } => self.node("ConstDeclaration", position, |p| {
                p.identifier(identifier);
                p.expression(expression);
            }),
            DeclarationKind::Func { identifier, parameters, return_type, body } => self.node("FuncDeclaration", position, |p| {
                p.identifier(identifier);
                p.formal_parameter_sequence(parameters);
                p.type_denoter(return_type);
                p.expression(body);
            }),
            DeclarationKind::Proc { identifier, parameters, body } => self.node("ProcDeclaration", position, |p| {
                p.identifier(identifier);
                p.formal_parameter_sequence(parameters);
                p.command(body);
            }),
            DeclarationKind::Sequential { first, second } => self.node("SequentialDeclaration", position, |p| {
                p.declaration(first);
                p.declaration(second);
            }),
            DeclarationKind::Type { identifier, type_denoter } => self.node("TypeDeclaration", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
            }),
            DeclarationKind::Var { identifier, type_denoter } => self.node("VarDeclaration", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
            }),
        }
    }

    fn formal_parameter(&mut self, parameter: &FormalParameter) {
        let position = &parameter.position;
        match &parameter.kind {
            FormalParameterKind::Const { identifier, type_denoter } => self.node("ConstFormalParameter", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
            }),
            FormalParameterKind::Func { identifier, parameters, return_type } => self.node("FuncFormalParameter", position, |p| {
                p.identifier(identifier);
                p.formal_parameter_sequence(parameters);
                p.type_denoter(return_type);
            }),
            FormalParameterKind::Proc { identifier, parameters } => self.node("ProcFormalParameter", position, |p| {
                p.identifier(identifier);
                p.formal_parameter_sequence(parameters);
            }),
            FormalParameterKind::Var { identifier, type_denoter } => self.node("VarFormalParameter", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
            }),
        }
    }

    fn formal_parameter_sequence(&mut self, sequence: &FormalParameterSequence) {
        let position = &sequence.position;
        match &*sequence.kind {
            FormalParameterSequenceKind::Empty => self.line("EmptyFormalParameterSequence", position),
            FormalParameterSequenceKind::Single(parameter) => self.node("SingleFormalParameterSequence", position, |p| p.formal_parameter(parameter)),
            FormalParameterSequenceKind::Multiple { parameter, rest } => self.node("MultipleFormalParameterSequence", position, |p| {
                p.formal_parameter(parameter);
                p.formal_parameter_sequence(rest);
            }),
        }
    }

    fn actual_parameter(&mut self, parameter: &ActualParameter) {
        let position = &parameter.position;
        match &parameter.kind {
            ActualParameterKind::Const(expression) => self.node("ConstActualParameter", position, |p| p.expression(expression)),
            ActualParameterKind::Func(identifier) => self.node("FuncActualParameter", position, |p| p.identifier(identifier)),
            ActualParameterKind::Proc(identifier) => self.node("ProcActualParameter", position, |p| p.identifier(identifier)),
            ActualParameterKind::Var(v_name) => self.node("VarActualParameter", position, |p| p.v_name(v_name)),
        }
    }

    fn actual_parameter_sequence(&mut self, sequence: &ActualParameterSequence) {
        let position = &sequence.position;
        match &*sequence.kind {
            ActualParameterSequenceKind::Empty => self.line("EmptyActualParameterSequence", position),
            ActualParameterSequenceKind::Single(parameter) => self.node("SingleActualParameterSequence", position, |p| p.actual_parameter(parameter)),
            ActualParameterSequenceKind::Multiple { parameter, rest } => self.node("MultipleActualParameterSequence", position, |p| {
                p.actual_parameter(parameter);
                p.actual_parameter_sequence(rest);
            }),
        }
    }

    fn type_denoter(&mut self, type_denoter: &TypeDenoter) {
        let position = &type_denoter.position;
        match &*type_denoter.kind {
            TypeDenoterKind::Array { size, element_type } => self.node("ArrayTypeDenoter", position, |p| {
                p.integer_literal(size);
                p.type_denoter(element_type);
            }),
            TypeDenoterKind::Record(fields) => self.node("RecordTypeDenoter", position, |p| p.field_type_denoter(fields)),
            TypeDenoterKind::Simple(identifier) => self.node("SimpleTypeDenoter", position, |p| p.identifier(identifier)),
        }
    }

    fn field_type_denoter(&mut self, field: &FieldTypeDenoter) {
        let position = &field.position;
        match &*field.kind {
            FieldTypeDenoterKind::Single { identifier, type_denoter } => self.node("SingleFieldTypeDenoter", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
            }),
            FieldTypeDenoterKind::Multiple { identifier, type_denoter, rest } => self.node("MultipleFieldTypeDenoter", position, |p| {
                p.identifier(identifier);
                p.type_denoter(type_denoter);
                p.field_type_denoter(rest);
            }),
        }
    }

    fn identifier(&mut self, identifier: &Identifier) {
        self.terminal("Identifier", identifier.spelling, &identifier.position);
    }

    fn integer_literal(&mut self, literal: &IntegerLiteral) {
        self.terminal("IntegerLiteral", literal.spelling, &literal.position);
    }

    fn character_literal(&mut self, literal: &CharacterLiteral) {
        self.terminal("CharacterLiteral", literal.spelling, &literal.position);
    }

    fn operator(&mut self, operator: &Operator) {
        self.terminal("Operator", operator.spelling, &operator.position);
    }
}
