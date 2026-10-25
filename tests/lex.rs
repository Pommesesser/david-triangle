use std::fs;
use std::path::Path;

use david_triangle::syntactic_analyzer::scanner::Scanner;
use david_triangle::syntactic_analyzer::source_stream::SourceStream;
use david_triangle::syntactic_analyzer::token_kind::TokenKind;

/// Lexes `source` in memory and renders it like the Java reference prints it:
/// one `test_repr` line per token, up to and including EOF.
fn lex(source: &[u8]) -> String {
    let mut scanner = Scanner::new(SourceStream::new(source));
    let mut output = String::new();
    loop {
        let token = scanner.scan();
        output.push_str(&token.test_repr());
        output.push('\n');
        if token.kind == TokenKind::Eof {
            return output;
        }
    }
}

/// Describes the first line where two outputs disagree.
fn first_difference(actual: &str, expected: &str) -> String {
    let mut actual_lines = actual.lines();
    let mut expected_lines = expected.lines();
    for line_number in 1.. {
        match (actual_lines.next(), expected_lines.next()) {
            (None, None) => return "same lines, different line endings".to_string(),
            (a, e) if a != e => return format!("line {line_number}: ours {a:?}, java {e:?}"),
            _ => {}
        }
    }
    unreachable!()
}

#[test]
fn lexer_matches_java_reference() {
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
        let expected = fs::read_to_string(tests_dir.join("expected/lex").join(format!("{name}.txt")))
            .unwrap_or_else(|_| panic!("missing expected/lex/{name}.txt"));

        let actual = lex(&source);

        if actual == expected {
            println!("PASS  {name:<28} {:>5} tokens", actual.lines().count());
        } else {
            println!("FAIL  {name}: {}", first_difference(&actual, &expected));
            failures.push(name.to_string());
        }
    }

    println!("{} of {} examples match the Java lexer", inputs.len() - failures.len(), inputs.len());
    assert!(failures.is_empty(), "differs from Java: {failures:?}");
}
