//! Development tooling for astral-yaml.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use astral_yaml::parse;
use xtask::{events, validate_spans};

const USAGE: &str = "usage: cargo xtask <conformance [suite-dir] | events file>";

fn cases(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            cases(&path, out)?;
        } else if path.file_name().is_some_and(|s| s == "in.yaml") {
            out.push(path);
        }
    }

    Ok(())
}

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    };

    if matches!(command.as_str(), "-h" | "--help") {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    if command == "events" {
        let source = fs::read_to_string(args.next().ok_or("expected a file path")?)?;
        let stream = parse(&source)?;
        validate_spans(&stream)?;
        print!("{}", events(&stream));
        return Ok(ExitCode::SUCCESS);
    }

    if command != "conformance" {
        return Err(USAGE.into());
    }

    let root = args.next().map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/yaml-test-suite"),
        PathBuf::from,
    );

    let mut paths = Vec::new();
    cases(&root, &mut paths)?;
    paths.sort();
    if paths.is_empty() {
        return Err("test suite contains no cases".into());
    }

    let mut passed = 0;
    let mut invalid = 0;

    for path in &paths {
        let dir = path.parent().ok_or("case has no directory")?;
        let id = dir.strip_prefix(&root)?.display();
        let source = fs::read_to_string(path)?;
        let expect_error = dir.join("error").exists();
        if expect_error {
            invalid += 1;
        }

        match parse(&source) {
            Err(_) if expect_error => passed += 1,
            Err(e) => println!("FAIL {id}: {e}"),
            Ok(stream) => {
                validate_spans(&stream)?;
                if expect_error {
                    println!("FAIL {id}: accepted malformed input");
                    continue;
                }

                let actual = events(&stream);
                let expected = fs::read_to_string(dir.join("test.event"))?;
                if actual == expected {
                    passed += 1;
                } else {
                    let line = actual
                        .lines()
                        .zip(expected.lines())
                        .position(|(a, b)| a != b)
                        .unwrap_or_else(|| actual.lines().count().min(expected.lines().count()));

                    println!(
                        "FAIL {id}: event {}: got {:?}, expected {:?}",
                        line + 1,
                        actual.lines().nth(line),
                        expected.lines().nth(line)
                    );
                }
            }
        }
    }

    println!(
        "{passed}/{} cases passed ({} valid, {invalid} invalid)",
        paths.len(),
        paths.len() - invalid
    );
    Ok(if passed == paths.len() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
