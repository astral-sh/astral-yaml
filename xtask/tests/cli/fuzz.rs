use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

use super::Directory;

// These stand-ins record real subprocess arguments without installing tools or
// fuzzing during `cargo test`. They need only POSIX shell builtins.
const TOOL: &str = r#"#!/bin/sh
case "${0##*/}:$1:$4" in
    uv:run:python) step=seed ;;
    uv:run:rustup) step=fuzz ;;
    rustup:run:rustc) step=nightly ;;
    rustup:component:nightly) step=components ;;
    *) exit 99 ;;
esac

{
    printf '%s\t%s\t%s\t%s' "$step" "$PWD" "${CARGO-unset}" "${RUSTC-unset}"
    printf '\t%s' "$@"
    printf '\n'
} >> "$XTASK_TEST_LOG"

if [ "$step" = "${XTASK_TEST_FAIL_STEP-}" ]; then
    exit 37
fi

if [ "$step" = fuzz ] && [ "${XTASK_TEST_SIGNAL-}" = yes ]; then
    kill -TERM "$$"
fi
"#;

fn install_tools(directory: &Directory) {
    for name in ["uv", "rustup"] {
        directory.write(name, TOOL);
        fs::set_permissions(directory.0.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn fuzz_command(directory: &Directory, args: &[&str]) -> Command {
    let mut command = directory.command(&["fuzz"]);
    command
        .args(args)
        .env("PATH", &directory.0)
        .env("XTASK_TEST_LOG", directory.0.join("calls"))
        .env("CARGO", "parent-cargo")
        .env("RUSTC", "parent-rustc");

    command
}

fn calls(directory: &Directory) -> Vec<Vec<String>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let log = fs::read_to_string(directory.0.join("calls")).unwrap();

    log.lines()
        .map(|line| {
            let fields: Vec<_> = line.split('\t').map(str::to_owned).collect();
            assert_eq!(Path::new(&fields[1]), root);
            assert_eq!(&fields[2..4], ["unset", "unset"]);

            fields
        })
        .collect()
}

#[test]
fn fuzz_sets_up_and_runs_all_targets_with_limits() {
    let directory = Directory::new();
    install_tools(&directory);
    let output = fuzz_command(&directory, &[]).output().unwrap();
    assert!(output.status.success());

    let calls = calls(&directory);
    assert_eq!(calls.len(), 7);
    assert_eq!(calls[0][0], "seed");
    assert_eq!(
        &calls[0][4..],
        [
            "run",
            "--only-dev",
            "--locked",
            "python",
            "scripts/seed-fuzz.py"
        ]
    );
    assert_eq!(calls[1][0], "nightly");
    assert_eq!(
        &calls[1][4..],
        ["run", "--install", "nightly", "rustc", "--version"]
    );
    assert_eq!(calls[2][0], "components");
    assert_eq!(
        &calls[2][4..],
        [
            "component",
            "add",
            "--toolchain",
            "nightly",
            "rust-src",
            "llvm-tools-preview"
        ]
    );

    for (call, target) in calls[3..].iter().zip(["parse", "serde", "decode", "typed"]) {
        assert_eq!(call[0], "fuzz");
        assert_eq!(
            &call[4..],
            [
                "run",
                "--only-dev",
                "--locked",
                "rustup",
                "run",
                "nightly",
                "cargo",
                "fuzz",
                "run",
                target,
                "--",
                "-max_total_time=60",
                "-timeout=2",
                "-max_len=65536",
                "-rss_limit_mb=2048",
            ]
        );
    }
}

#[test]
fn fuzz_forwards_subcommands_and_arguments_without_shell_expansion() {
    let directory = Directory::new();
    install_tools(&directory);
    let args = [
        "run",
        "typed",
        "corpus with spaces",
        "--",
        "-runs=7",
        "-dict=$literal;name",
    ];
    let output = fuzz_command(&directory, &args).output().unwrap();
    assert!(output.status.success());

    let calls = calls(&directory);
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[3][0], "fuzz");
    assert_eq!(&calls[3][12..], args);
}

#[test]
fn fuzz_stops_on_setup_or_target_failure_and_preserves_exit_codes() {
    for (index, step) in ["seed", "nightly", "components", "fuzz"].iter().enumerate() {
        let directory = Directory::new();
        install_tools(&directory);
        let output = fuzz_command(&directory, &[])
            .env("XTASK_TEST_FAIL_STEP", step)
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(37));
        let calls = calls(&directory);
        assert_eq!(calls.len(), index + 1);
        assert_eq!(&calls[index][0], step);
    }
}

#[test]
fn fuzz_reports_signalled_children_as_failures() {
    let directory = Directory::new();
    install_tools(&directory);
    let output = fuzz_command(&directory, &["run", "parse", "--", "-runs=1"])
        .env("XTASK_TEST_SIGNAL", "yes")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(calls(&directory).len(), 4);
}
