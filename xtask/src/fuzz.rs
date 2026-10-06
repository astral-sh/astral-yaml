use std::{
    ffi::OsString,
    io,
    path::Path,
    process::{Command, ExitCode},
};

fn command(root: &Path, program: &str, args: &[&str]) -> Command {
    let mut command = Command::new(program);
    command.current_dir(root).args(args);

    // Cargo can pass its own compiler and executable to `cargo run` children.
    // Let rustup select both from nightly for the fuzz build instead.
    command.env_remove("CARGO").env_remove("RUSTC");

    command
}

fn execute(command: &mut Command) -> io::Result<ExitCode> {
    let status = command.status().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to run {}: {error}",
                command.get_program().to_string_lossy()
            ),
        )
    })?;

    Ok(status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from))
}

pub(crate) fn run(args: Vec<OsString>) -> io::Result<ExitCode> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut setup = [
        command(&root, "uv", &["run", "--script", "scripts/seed-fuzz.py"]),
        command(
            &root,
            "rustup",
            &["run", "--install", "nightly", "rustc", "--version"],
        ),
        command(
            &root,
            "rustup",
            &[
                "component",
                "add",
                "--toolchain",
                "nightly",
                "rust-src",
                "llvm-tools-preview",
            ],
        ),
    ];

    for command in &mut setup {
        let code = execute(command)?;

        if code != ExitCode::SUCCESS {
            return Ok(code);
        }
    }

    let runs = if args.is_empty() {
        [
            "parse",
            "serde",
            "decode",
            "typed",
            "editable",
            "editable-model",
        ]
        .map(|target| {
            [
                "run",
                target,
                "--",
                "-max_total_time=60",
                "-timeout=2",
                "-max_len=65536",
                "-rss_limit_mb=2048",
            ]
            .map(OsString::from)
            .to_vec()
        })
        .to_vec()
    } else {
        vec![args]
    };

    for args in runs {
        let mut fuzz = command(
            &root,
            "uv",
            &[
                "run",
                "--only-dev",
                "--locked",
                "rustup",
                "run",
                "nightly",
                "cargo",
                "fuzz",
            ],
        );
        fuzz.args(args);
        let code = execute(&mut fuzz)?;

        if code != ExitCode::SUCCESS {
            return Ok(code);
        }
    }

    Ok(ExitCode::SUCCESS)
}
