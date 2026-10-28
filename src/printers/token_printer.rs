use crate::syntactic_analyzer::token::{Token, latin1_to_string};

// Same output as LexemePrinter.java in the Java reference:
//   <kind spelling>,<token spelling>,(<start>|<finish>)

pub fn print(token: &Token) -> String {
    format!(
        "{},{},({}|{})",
        token.kind.spelling(),
        latin1_to_string(token.spelling),
        token.position.start,
        token.position.finish
    )
}
