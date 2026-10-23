use crate::syntactic_analyzer::source_position::SourcePosition;

pub enum AST {
    Terminal(Terminal),
    Declaration(Declaration),
    FormalParameterSequence(FormalParameterSequence),
    ActualParameterSequence(ActualParameterSequence),
    Program,
    Expression(Expression),
    ActualParameter(ActualParameter),
    VName(VName),
    RecordAggregate(RecordAggregate),
    TypeDenoter(TypeDenoter),
    ArrayAggregate(ArrayAggregate),
    Command(Command),
}

pub enum Terminal {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    CharacterLiteral(CharacterLiteral),
}
pub struct Identifier{
spelling: String,
position: SourcePosition,
}
pub struct CharacterLiteral{
spelling: String,
position: SourcePosition,
}
pub struct IntegerLiteral{
spelling: String,
position: SourcePosition,
}

pub enum Declaration {
    VarDeclaration,
    ConstDeclaration,
    FormalParameter(FormalParameter),
    ProcDeclaration,
    FuncDeclaration,
    BinaryOperatorDeclaration,
    TypeDeclaration,
    SequentialDeclaration,
    UnaryOperatorDeclaration,
}
pub enum FormalParameter {
    ConstFormalParameter,
    VarFormalParameter,
    ProcFormalParameter,
    FuncFormalParameter,
}
pub enum FormalParameterSequence {
    MultipleFormalParameterSequence,
    EmptyFormalParameterSequence,
    SingleFormalParameterSequence,
}
pub enum ActualParameterSequence {
    MultipleFormalParameterSequence,
    EmptyActualParameterSequence,
    SingleActualParameterSequence,
}
pub enum Expression {
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    IntegerExpression,
    VNameExpression,
    LetExpression,
    CharacterExpression,
    ArrayExpression,
    RecordExpression,
    EmptyExpression,
    IfExpression,
}
pub enum ActualParameter {
    ProcActualParameter,
    ConstActualParameter,
    FuncActualParameter,
    VarActualParameter,
}
pub enum VName {
    SubscriptVName,
    DotVName,
    SimpleVName,
}
pub enum RecordAggregate {
    SingleRecordAggregate,
    MultipleRecordAggregate,
}
pub enum TypeDenoter {
    SimpleTypeDenoter,
    CharTypeDenoter,
    RecordTypeDenoter,
    ErrorTypeDenoter,
    ArrayTypeDenoter,
    IntTypeDenoter,
    BoolTypeDenoter,
    AnyTypeDenoter,
    FieldTypeDenoter(FieldTypeDenoter)
}
pub enum FieldTypeDenoter {
    MultipleFieldTypeDenoter,
    SingleFieldTypeDenoter,
}
pub enum ArrayAggregate {
    SingleArrayAggregate,
    MultipleArrayAggregate,
}
pub enum Command {
    LetCommand{position: SourcePosition, declaration: Declaration, command: Box<Command>},
    CallCommand,
    WhileCommand,
    SequentialCommand,
    EmptyCommand,
    IfCommand,
    AssignCommand,
}