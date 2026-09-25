//! Development tooling for astral-yaml.
mod fuzz;

use std::{env, process::ExitCode};

const USAGE: &str = "usage: cargo xtask fuzz [cargo-fuzz arguments...]\n\nWithout arguments, fuzz runs parse, serde, decode, and typed for 60 seconds each.";

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

    if command != "fuzz" {
        return Err(USAGE.into());
    }

    Ok(fuzz::run(args.collect())?)
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
