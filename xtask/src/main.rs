//! Development tooling for astral-yaml.
mod cov;
mod fuzz;

use std::{env, process::ExitCode};

const USAGE: &str = "usage: cargo xtask <command>

Commands:
  cov [--html]                   Run coverage and enforce 100% lines and functions
  fuzz [cargo-fuzz arguments...]  Run all four targets for 60 seconds each, or forward arguments";

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

    if command == "cov" {
        cov::run(&args.collect::<Vec<_>>())?;
        return Ok(ExitCode::SUCCESS);
    }

    if command == "fuzz" {
        return Ok(fuzz::run(args.collect())?);
    }

    Err(USAGE.into())
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
