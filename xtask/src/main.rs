//! Development tooling for astral-yaml.
mod fuzz;

use std::{env, fs, process::ExitCode};

use astral_yaml::parse;
use xtask::{events, validate_spans};

const USAGE: &str = "usage: cargo xtask <events file | fuzz [cargo-fuzz arguments...]>\n\nWithout arguments, fuzz runs parse, serde, decode, and typed for 60 seconds each.";

fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    };

    if command == "-h" || command == "--help" {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    }

    if command == "fuzz" {
        return Ok(fuzz::run(args.collect())?);
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
