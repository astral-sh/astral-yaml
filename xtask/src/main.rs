//! Development tooling for astral-yaml.
mod cov;
mod fuzz;

use std::{ffi::OsString, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(bin_name = "cargo xtask", about, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Require 100% line and function coverage for the YAML crates.
    Cov {
        /// Write an HTML report, including when coverage falls below the threshold.
        #[arg(long)]
        html: bool,
    },

    /// Run all four fuzz targets for 60 seconds each, or forward arguments to cargo-fuzz.
    Fuzz {
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            value_name = "CARGO_FUZZ_ARGS"
        )]
        args: Vec<OsString>,
    },
}

fn run(command: Command) -> Result<ExitCode, Box<dyn std::error::Error>> {
    match command {
        Command::Cov { html } => {
            cov::run(html)?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Fuzz { args } => Ok(fuzz::run(args)?),
    }
}

fn main() -> ExitCode {
    match run(Cli::parse().command) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
