pub enum AST {
    Terminal(Terminal),
    Declaration(Declaration),
    FormalParameterSequence(FormalParameterSequence),
    ActualParameterSequence(ActualParameterSequence),
    Program(Program),
    Expression(Expression),
    ActualParameter(ActualParameter),
    VName(VName),
    RecordAggregate(RecordAggregate),
    TypeDenoter(TypeDenoter),
    ArrayAggregate(ArrayAggregate),
    Command(Command),
}

pub enum Terminal {
    Identifier,
    CharacterLiteral,
    IntegerLiteral,
}
pub enum Declaration {
    
}
pub enum FormalParameterSequence {}
pub enum ActualParameterSequence {}
pub enum Program {}
pub enum Expression {}
pub enum ActualParameter {}
pub enum VName {}
pub enum RecordAggregate {}
pub enum TypeDenoter {}
pub enum ArrayAggregate {}
pub enum Command {}