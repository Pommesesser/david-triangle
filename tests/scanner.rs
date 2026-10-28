use david_triangle::printers::token_printer::print;
use david_triangle::syntactic_analyzer::scanner::scan_all;
use std::fs;
use std::path::Path;

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
fn scanner_matches_java_reference() {
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
        let expected = fs::read_to_string(
            tests_dir
                .join("expected/scanner")
                .join(format!("{name}.txt")),
        )
        .unwrap_or_else(|_| panic!("missing expected/scanner/{name}.txt"));

        let tokens = scan_all(&source);

        let mut actual = String::new();
        for token in tokens {
            actual.push_str(&print(&token));
            actual.push('\n');
        }

        if actual == expected {
            println!("PASS  {name:<28} {:>5} tokens", actual.lines().count());
        } else {
            println!("FAIL  {name}: {}", first_difference(&actual, &expected));
            failures.push(name.to_string());
        }
    }

    println!(
        "{} of {} examples match the Java scanner",
        inputs.len() - failures.len(),
        inputs.len()
    );
    assert!(failures.is_empty(), "differs from Java: {failures:?}");
}
