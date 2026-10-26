use std::fs;
use std::panic;
use std::path::Path;

use david_triangle::abstract_syntax_trees::ast::Program;
use david_triangle::printers::program_printer;
use david_triangle::syntactic_analyzer::parser::Parser;
use david_triangle::syntactic_analyzer::scanner::Scanner;
use david_triangle::syntactic_analyzer::source_stream::SourceStream;

fn parse(source: &[u8]) -> Program {
    Parser::new(Scanner::new(SourceStream::new(source))).parse_program()
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