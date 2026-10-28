use crate::syntactic_analyzer::source_position::SourcePosition;

// Translation of Triangle.AbstractSyntaxTrees, restricted to the nodes the parser builds.
// Every abstract Java class becomes a `struct X { kind: XKind, position }` wrapper around an
// enum of its concrete subclasses. Recursive nodes point to their `kind` in the parser's arena, which
// breaks every cycle; the whole tree is freed at once when the arena is dropped.

// Terminals

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier<'a> {
    pub spelling: &'a [u8],
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegerLiteral<'a> {
    pub spelling: &'a [u8],
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterLiteral<'a> {
    pub spelling: &'a [u8],
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator<'a> {
    pub spelling: &'a [u8],
    pub position: SourcePosition,
}

// Program

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program<'a> {
    pub command: Command<'a>,
    pub position: SourcePosition,
}

// Commands

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command<'a> {
    pub kind: &'a CommandKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandKind<'a> {
    Assign {
        v_name: VName<'a>,
        expression: Expression<'a>,
    },
    Call {
        identifier: Identifier<'a>,
        arguments: ActualParameterSequence<'a>,
    },
    Empty,
    If {
        condition: Expression<'a>,
        then_branch: Command<'a>,
        else_branch: Command<'a>,
    },
    Let {
        declaration: Declaration<'a>,
        body: Command<'a>,
    },
    Sequential {
        first: Command<'a>,
        second: Command<'a>,
    },
    While {
        condition: Expression<'a>,
        body: Command<'a>,
    },
}

// Expressions

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression<'a> {
    pub kind: &'a ExpressionKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionKind<'a> {
    Array(ArrayAggregate<'a>),
    Binary {
        left: Expression<'a>,
        operator: Operator<'a>,
        right: Expression<'a>,
    },
    Call {
        identifier: Identifier<'a>,
        arguments: ActualParameterSequence<'a>,
    },
    Character(CharacterLiteral<'a>),
    Empty,
    If {
        condition: Expression<'a>,
        then_branch: Expression<'a>,
        else_branch: Expression<'a>,
    },
    Integer(IntegerLiteral<'a>),
    Let {
        declaration: Declaration<'a>,
        body: Expression<'a>,
    },
    Record(RecordAggregate<'a>),
    Unary {
        operator: Operator<'a>,
        operand: Expression<'a>,
    },
    VName(VName<'a>),
}

// Aggregates

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayAggregate<'a> {
    pub kind: &'a ArrayAggregateKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrayAggregateKind<'a> {
    Single(Expression<'a>),
    Multiple {
        expression: Expression<'a>,
        rest: ArrayAggregate<'a>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordAggregate<'a> {
    pub kind: &'a RecordAggregateKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordAggregateKind<'a> {
    Single {
        identifier: Identifier<'a>,
        expression: Expression<'a>,
    },
    Multiple {
        identifier: Identifier<'a>,
        expression: Expression<'a>,
        rest: RecordAggregate<'a>,
    },
}

// Value-or-variable names

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VName<'a> {
    pub kind: &'a VNameKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VNameKind<'a> {
    Simple(Identifier<'a>),
    Dot {
        v_name: VName<'a>,
        field: Identifier<'a>,
    },
    Subscript {
        v_name: VName<'a>,
        index: Expression<'a>,
    },
}

// Declarations

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration<'a> {
    pub kind: &'a DeclarationKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclarationKind<'a> {
    Const {
        identifier: Identifier<'a>,
        expression: Expression<'a>,
    },
    Func {
        identifier: Identifier<'a>,
        parameters: FormalParameterSequence<'a>,
        return_type: TypeDenoter<'a>,
        body: Expression<'a>,
    },
    Proc {
        identifier: Identifier<'a>,
        parameters: FormalParameterSequence<'a>,
        body: Command<'a>,
    },
    Sequential {
        first: Declaration<'a>,
        second: Declaration<'a>,
    },
    Type {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
    },
    Var {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
    },
}

// Formal parameters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalParameter<'a> {
    pub kind: FormalParameterKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalParameterKind<'a> {
    Const {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
    },
    Func {
        identifier: Identifier<'a>,
        parameters: FormalParameterSequence<'a>,
        return_type: TypeDenoter<'a>,
    },
    Proc {
        identifier: Identifier<'a>,
        parameters: FormalParameterSequence<'a>,
    },
    Var {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormalParameterSequence<'a> {
    pub kind: &'a FormalParameterSequenceKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormalParameterSequenceKind<'a> {
    Empty,
    Single(FormalParameter<'a>),
    Multiple {
        parameter: FormalParameter<'a>,
        rest: FormalParameterSequence<'a>,
    },
}

// Actual parameters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActualParameter<'a> {
    pub kind: ActualParameterKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualParameterKind<'a> {
    Const(Expression<'a>),
    Func(Identifier<'a>),
    Proc(Identifier<'a>),
    Var(VName<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActualParameterSequence<'a> {
    pub kind: &'a ActualParameterSequenceKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActualParameterSequenceKind<'a> {
    Empty,
    Single(ActualParameter<'a>),
    Multiple {
        parameter: ActualParameter<'a>,
        rest: ActualParameterSequence<'a>,
    },
}

// Type denoters

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDenoter<'a> {
    pub kind: &'a TypeDenoterKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDenoterKind<'a> {
    Array {
        size: IntegerLiteral<'a>,
        element_type: TypeDenoter<'a>,
    },
    Record(FieldTypeDenoter<'a>),
    Simple(Identifier<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldTypeDenoter<'a> {
    pub kind: &'a FieldTypeDenoterKind<'a>,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTypeDenoterKind<'a> {
    Single {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
    },
    Multiple {
        identifier: Identifier<'a>,
        type_denoter: TypeDenoter<'a>,
        rest: FieldTypeDenoter<'a>,
    },
}
