use crate::syntactic_analyzer::source_position::SourcePosition;

// Translation of Triangle.AbstractSyntaxTrees, restricted to the nodes the parser builds.
// Every abstract Java class becomes a `struct X { kind: XKind, position }` wrapper around an
// enum of its concrete subclasses. Recursive nodes box their `kind`, which breaks every cycle with a single allocation per node.

// Terminals

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub spelling: String,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLiteral {
    pub spelling: String,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterLiteral {
    pub spelling: String,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
    pub spelling: String,
    pub position: SourcePosition,
}

// Program

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub command: Command,
    pub position: SourcePosition,
}

// Commands

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub kind: Box<CommandKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind {
    Assign {
        v_name: VName,
        expression: Expression,
    },
    Call {
        identifier: Identifier,
        arguments: ActualParameterSequence,
    },
    Empty,
    If {
        condition: Expression,
        then_branch: Command,
        else_branch: Command,
    },
    Let {
        declaration: Declaration,
        body: Command,
    },
    Sequential {
        first: Command,
        second: Command,
    },
    While {
        condition: Expression,
        body: Command,
    },
}

// Expressions

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub kind: Box<ExpressionKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind {
    Array(ArrayAggregate),
    Binary {
        left: Expression,
        operator: Operator,
        right: Expression,
    },
    Call {
        identifier: Identifier,
        arguments: ActualParameterSequence,
    },
    Character(CharacterLiteral),
    Empty,
    If {
        condition: Expression,
        then_branch: Expression,
        else_branch: Expression,
    },
    Integer(IntegerLiteral),
    Let {
        declaration: Declaration,
        body: Expression,
    },
    Record(RecordAggregate),
    Unary {
        operator: Operator,
        operand: Expression,
    },
    VName(VName),
}

// Aggregates

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayAggregate {
    pub kind: Box<ArrayAggregateKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayAggregateKind {
    Single(Expression),
    Multiple {
        expression: Expression,
        rest: ArrayAggregate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAggregate {
    pub kind: Box<RecordAggregateKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordAggregateKind {
    Single {
        identifier: Identifier,
        expression: Expression,
    },
    Multiple {
        identifier: Identifier,
        expression: Expression,
        rest: RecordAggregate,
    },
}

// Value-or-variable names

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VName {
    pub kind: Box<VNameKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VNameKind {
    Simple(Identifier),
    Dot {
        v_name: VName,
        field: Identifier,
    },
    Subscript {
        v_name: VName,
        index: Expression,
    },
}

// Declarations

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub kind: Box<DeclarationKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationKind {
    Const {
        identifier: Identifier,
        expression: Expression,
    },
    Func {
        identifier: Identifier,
        parameters: FormalParameterSequence,
        return_type: TypeDenoter,
        body: Expression,
    },
    Proc {
        identifier: Identifier,
        parameters: FormalParameterSequence,
        body: Command,
    },
    Sequential {
        first: Declaration,
        second: Declaration,
    },
    Type {
        identifier: Identifier,
        type_denoter: TypeDenoter,
    },
    Var {
        identifier: Identifier,
        type_denoter: TypeDenoter,
    },
}

// Formal parameters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalParameter {
    pub kind: FormalParameterKind,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalParameterKind {
    Const {
        identifier: Identifier,
        type_denoter: TypeDenoter,
    },
    Func {
        identifier: Identifier,
        parameters: FormalParameterSequence,
        return_type: TypeDenoter,
    },
    Proc {
        identifier: Identifier,
        parameters: FormalParameterSequence,
    },
    Var {
        identifier: Identifier,
        type_denoter: TypeDenoter,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalParameterSequence {
    pub kind: Box<FormalParameterSequenceKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalParameterSequenceKind {
    Empty,
    Single(FormalParameter),
    Multiple {
        parameter: FormalParameter,
        rest: FormalParameterSequence,
    },
}

// Actual parameters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActualParameter {
    pub kind: ActualParameterKind,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualParameterKind {
    Const(Expression),
    Func(Identifier),
    Proc(Identifier),
    Var(VName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActualParameterSequence {
    pub kind: Box<ActualParameterSequenceKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualParameterSequenceKind {
    Empty,
    Single(ActualParameter),
    Multiple {
        parameter: ActualParameter,
        rest: ActualParameterSequence,
    },
}

// Type denoters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDenoter {
    pub kind: Box<TypeDenoterKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDenoterKind {
    Array {
        size: IntegerLiteral,
        element_type: TypeDenoter,
    },
    Record(FieldTypeDenoter),
    Simple(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldTypeDenoter {
    pub kind: Box<FieldTypeDenoterKind>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTypeDenoterKind {
    Single {
        identifier: Identifier,
        type_denoter: TypeDenoter,
    },
    Multiple {
        identifier: Identifier,
        type_denoter: TypeDenoter,
        rest: FieldTypeDenoter,
    },
}
