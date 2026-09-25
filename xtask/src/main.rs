//! Development tooling for astral-yaml.
use std::{env, fs, process::ExitCode};

use astral_yaml::parse;
use xtask::{events, validate_spans};

const USAGE: &str = "usage: cargo xtask events file";

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

    if command != "events" {
        return Err(USAGE.into());
    }

    let source = fs::read_to_string(args.next().ok_or("expected a file path")?)?;
    let stream = parse(&source)?;
    validate_spans(&stream)?;
    print!("{}", events(&stream));

    Ok(ExitCode::SUCCESS)
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
