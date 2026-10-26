use std::fs;
use std::panic;
use std::path::Path;

use david_triangle::abstract_syntax_trees::ast::{CommandKind, ExpressionKind, Program};
use david_triangle::printers::program_printer;
use david_triangle::syntactic_analyzer::parser::Parser;
use david_triangle::syntactic_analyzer::scanner::Scanner;
use david_triangle::syntactic_analyzer::source_stream::SourceStream;

fn parse(source: &[u8]) -> Program {
    Parser::new(Scanner::new(SourceStream::new(source))).parse_program()
}

fn parse_error(source: &'static str) -> String {
    let error = panic::catch_unwind(|| parse(source.as_bytes())).expect_err("expected a syntax error");
    error.downcast_ref::<String>().cloned().expect("panic payload should be a String")
}

fn first_difference(actual: &str, expected: &str) -> String {
    let mut actual_lines = actual.lines();
    let mut expected_lines = expected.lines();
    for line_number in 1.. {
        match (actual_lines.next(), expected_lines.next()) {
            (None, None) => return "same lines, different line endings".to_string(),
            (a, e) if a != e => return format!("line {line_number}: me {a:?}, java {e:?}"),
            _ => {}
        }
    }
    unreachable!()
}

#[test]
fn parser_matches_java_reference() {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");

    let mut inputs: Vec<_> = fs::read_dir(tests_dir.join("examples"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "tri"))
        .collect();
    inputs.sort();
    // Guard against a wrong path making the test pass by checking nothing.
    assert!(!inputs.is_empty(), "no example programs found");

    let mut failures = Vec::new();
    for input in &inputs {
        let name = input.file_stem().unwrap().to_str().unwrap();

        let source = fs::read(input).unwrap();
        let expected = fs::read_to_string(tests_dir.join("expected/parse").join(format!("{name}.txt")))
            .unwrap_or_else(|_| panic!("missing expected/parse/{name}.txt"));

        // A syntax error panics with the same "ERROR: ..." line the Java parser prints.
        let actual = match panic::catch_unwind(|| program_printer::print(&parse(&source))) {
            Ok(tree) => tree,
            Err(error) => format!("{}\n", error.downcast_ref::<String>().unwrap()),
        };

        if actual == expected {
            println!("PASS  {name:<28} {:>5} nodes", actual.lines().count());
        } else {
            println!("FAIL  {name}: {}", first_difference(&actual, &expected));
            failures.push(name.to_string());
        }
    }

    println!("{} of {} examples match the Java parser", inputs.len() - failures.len(), inputs.len());
    assert!(failures.is_empty(), "differs from Java: {failures:?}");
}

#[test]
fn first_token_is_not_skipped() {
    let program = parse(b"x := 1");
    assert!(matches!(*program.command.kind, CommandKind::Assign { .. }));
}

#[test]
fn binary_operators_are_left_associative() {
    let program = parse(b"x := 1 - 2 - 3");
    let CommandKind::Assign { expression, .. } = *program.command.kind else { panic!("expected assign") };
    let ExpressionKind::Binary { left, operator, .. } = *expression.kind else { panic!("expected binary") };
    assert_eq!(operator.spelling, "-");
    assert!(matches!(*left.kind, ExpressionKind::Binary { .. }));
}

#[test]
fn reports_errors_like_java() {
    assert!(parse_error("x := ").contains("\"\" cannot start an expression"));
    assert!(parse_error("x 1").contains("\":=\" expected here"));
    assert!(parse_error("let in x := 1").contains("\"in\" cannot start a declaration"));
    assert!(parse_error("begin x := 1 end end").contains("\"end\" not expected after end of program"));
}
