use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};

use serde_json::json;

use super::Directory;

// Exercise the actual CLI and filesystem without recursively running cargo test.
const TOOL: &str = r#"#!/bin/sh
step=${0##*/}
case "$step:$1" in
    cargo:metadata) step=metadata ;;
    cargo:test)
        step=tests
        for arg in "$@"; do
            if [ "$arg" = --no-run ]; then step=build; fi
        done
        ;;
    rustc:-vV) step=version ;;
    rustc:--print) step=sysroot ;;
    llvm-cov:*) step=$1 ;;
esac

{
    printf '%s\t%s\t%s\t%s\t%s' "$step" "$PWD" "${CARGO_ENCODED_RUSTFLAGS-}" "${LLVM_PROFILE_FILE-}" "${CARGO_INCREMENTAL-}"
    printf '\t%s' "$@"
    printf '\n'
} >> "$XTASK_TEST_DIR/calls"

if [ "$step" = "${XTASK_TEST_FAIL_STEP-}" ]; then exit 37; fi

emit() {
    while IFS= read -r line; do printf '%s\n' "$line"; done < "$XTASK_TEST_DIR/$1"
}

case "$step" in
    metadata) emit metadata.json ;;
    version) printf '%s\n' "${XTASK_TEST_VERSION-host: test-host}" ;;
    sysroot) printf '%s/sysroot\n' "$XTASK_TEST_DIR" ;;
    build) emit artifacts.json ;;
    tests)
        if [ "${XTASK_TEST_NO_PROFILES-}" != yes ]; then
            printf profile > "$XTASK_TEST_DIR/target/coverage/profiles/new.profraw"
        fi
        printf unrelated > "$XTASK_TEST_DIR/target/coverage/profiles/notes.txt"
        ;;
    llvm-profdata) printf merged > "$4" ;;
    report) printf 'Coverage report\n' ;;
    export) emit summary.json ;;
    show)
        while [ "$1" != -output-dir ]; do shift; done
        printf '<html>Coverage report</html>\n' > "$2/index.html"
        ;;
    rustup)
        if [ "${XTASK_TEST_INSTALL-}" = yes ]; then
            /bin/cp "$XTASK_TEST_DIR/llvm-cov" "$XTASK_TEST_DIR/sysroot/lib/rustlib/test-host/bin/llvm-cov"
            /bin/cp "$XTASK_TEST_DIR/llvm-profdata" "$XTASK_TEST_DIR/sysroot/lib/rustlib/test-host/bin/llvm-profdata"
        fi
        ;;
    *) exit 99 ;;
esac
"#;

fn setup() -> Directory {
    let directory = Directory::new();

    for path in [
        "sysroot/lib/rustlib/test-host/bin",
        "workspace with spaces/src/nested",
        "workspace with spaces/src/tests",
        "target/coverage/profiles",
        "target/coverage/html",
    ] {
        fs::create_dir_all(directory.0.join(path)).unwrap();
    }

    for name in [
        "cargo",
        "rustc",
        "rustup",
        "llvm-cov",
        "llvm-profdata",
        "sysroot/lib/rustlib/test-host/bin/llvm-cov",
        "sysroot/lib/rustlib/test-host/bin/llvm-profdata",
    ] {
        directory.write(name, TOOL);
        fs::set_permissions(directory.0.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }

    for path in [
        "workspace with spaces/src/lib.rs",
        "workspace with spaces/src/nested/code.rs",
        "workspace with spaces/src/tests.rs",
        "workspace with spaces/src/tests/support.rs",
        "workspace with spaces/src/README.md",
        "target/coverage/profiles/stale.profraw",
        "target/coverage/html/stale.html",
    ] {
        directory.write(path, "stale\n");
    }

    directory.write(
        "metadata.json",
        &format!(
            "{}\n",
            json!({
                "workspace_members": ["workspace"],
                "packages": [
                    {"id": "workspace", "manifest_path": directory.0.join("workspace with spaces/Cargo.toml")},
                    {"id": "dependency", "manifest_path": "/dependency/Cargo.toml"}
                ],
                "target_directory": directory.0.join("target")
            })
        ),
    );
    directory.write(
        "artifacts.json",
        &[
            json!({"reason": "compiler-artifact", "package_id": "workspace", "executable": "/build/unit tests", "profile": {"test": true}}),
            json!({"reason": "compiler-artifact", "package_id": "workspace", "executable": "/build/xtask", "profile": {"test": false}}),
            json!({"reason": "compiler-artifact", "package_id": "workspace", "executable": "/build/xtask"}),
            json!({"reason": "compiler-artifact", "package_id": "dependency", "executable": "/dependency/build-script"}),
            json!({"reason": "compiler-artifact", "package_id": "workspace", "executable": null}),
            json!({"reason": "build-finished", "success": true}),
        ]
        .map(|value| format!("{value}\n"))
        .concat(),
    );
    directory.write("summary.json", &summary(10, 10, 3, 3));

    directory
}

fn summary(lines: u64, lines_covered: u64, functions: u64, functions_covered: u64) -> String {
    format!(
        "{}\n",
        json!({"data": [{"totals": {
            "lines": {"count": lines, "covered": lines_covered},
            "functions": {"count": functions, "covered": functions_covered}
        }}]})
    )
}

fn command(directory: &Directory, args: &[&str]) -> Command {
    let mut command = directory.command(&["cov"]);
    command
        .args(args)
        .env("XTASK_TEST_DIR", &directory.0)
        .env("PATH", &directory.0)
        .env("CARGO", directory.0.join("cargo"))
        .env("RUSTC", directory.0.join("rustc"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");

    command
}

fn calls(directory: &Directory) -> Vec<Vec<String>> {
    fs::read_to_string(directory.0.join("calls"))
        .unwrap()
        .lines()
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect()
}

fn steps(calls: &[Vec<String>]) -> Vec<&str> {
    calls.iter().map(|call| call[0].as_str()).collect()
}

fn assert_instrumented_build(directory: &Directory, calls: &[Vec<String>]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();

    for call in &calls[..5] {
        assert_eq!(PathBuf::from(&call[1]), root);
    }

    assert_eq!(
        &calls[0][5..],
        ["metadata", "--format-version=1", "--no-deps", "--locked"]
    );
    assert_eq!(&calls[1][5..], ["-vV"]);
    assert_eq!(&calls[2][5..], ["--print", "sysroot"]);

    for call in &calls[3..5] {
        assert_eq!(call[2], "-C\x1finstrument-coverage");
        assert_eq!(
            PathBuf::from(&call[3]),
            directory.0.join("target/coverage/profiles/%m-%p.profraw")
        );
        assert_eq!(call[4], "0");
        assert_eq!(
            &call[5..12],
            [
                "test",
                "--workspace",
                "--tests",
                "--locked",
                "--target",
                "test-host",
                "--target-dir"
            ]
        );
        assert_eq!(
            PathBuf::from(&call[12]),
            directory.0.join("target/coverage/build")
        );
    }

    assert_eq!(
        &calls[3][13..],
        ["--no-run", "--message-format=json-render-diagnostics"]
    );
    assert_eq!(calls[4].len(), 13);
}

#[test]
fn cov_collects_current_binaries_and_fresh_profiles_for_both_reports() {
    let directory = setup();
    let output = command(&directory, &["--html"]).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let calls = calls(&directory);
    assert_eq!(
        steps(&calls),
        [
            "metadata",
            "version",
            "sysroot",
            "build",
            "tests",
            "llvm-profdata",
            "report",
            "show",
            "export"
        ]
    );
    assert_instrumented_build(&directory, &calls);

    assert_eq!(&calls[5][5..8], ["merge", "-sparse", "-o"]);
    assert_eq!(calls[5].len(), 10);
    assert_eq!(
        PathBuf::from(&calls[5][9]),
        directory.0.join("target/coverage/profiles/new.profraw")
    );
    assert!(
        !directory
            .0
            .join("target/coverage/profiles/stale.profraw")
            .exists()
    );

    let report_args = &calls[6][6..];
    assert_eq!(&calls[7][10..], report_args);
    assert_eq!(&calls[8][7..], report_args);
    assert_eq!(
        &report_args[2..7],
        [
            "-object",
            "/build/unit tests",
            "-object",
            "/build/xtask",
            "-sources"
        ]
    );
    assert_eq!(report_args.len(), 9);
    assert_eq!(
        PathBuf::from(&report_args[7]),
        directory.0.join("workspace with spaces/src/lib.rs")
    );
    assert_eq!(
        PathBuf::from(&report_args[8]),
        directory.0.join("workspace with spaces/src/nested/code.rs")
    );
    assert_eq!(
        &calls[7][5..9],
        [
            "show",
            "-format=html",
            "-show-line-counts-or-regions",
            "-output-dir"
        ]
    );
    assert!(!directory.0.join("target/coverage/html/stale.html").exists());
    assert!(
        directory
            .0
            .join("target/coverage/html/index.html")
            .is_file()
    );
    assert_eq!(
        fs::read_to_string(directory.0.join("target/coverage/summary.json")).unwrap(),
        summary(10, 10, 3, 3)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("target/coverage/html/index.html"));
}

#[test]
fn cov_preserves_compiler_flags_and_encoded_flags_take_precedence() {
    for (flags, encoded, expected) in [
        (None, None, "-C\x1finstrument-coverage"),
        (
            Some("--cfg example -C debuginfo=2"),
            None,
            "--cfg\x1fexample\x1f-C\x1fdebuginfo=2\x1f-C\x1finstrument-coverage",
        ),
        (
            Some("ignored"),
            Some("--cfg\x1fvalue=\"with spaces\""),
            "--cfg\x1fvalue=\"with spaces\"\x1f-C\x1finstrument-coverage",
        ),
    ] {
        let directory = setup();
        let mut command = command(&directory, &[]);

        if let Some(flags) = flags {
            command.env("RUSTFLAGS", flags);
        }

        if let Some(encoded) = encoded {
            command.env("CARGO_ENCODED_RUSTFLAGS", encoded);
        }

        let output = command.output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let calls = calls(&directory);
        assert!(!steps(&calls).contains(&"show"));
        assert_eq!(calls[3][2], expected);
        assert_eq!(calls[4][2], expected);
    }
}

#[test]
fn cov_generates_html_before_rejecting_incomplete_or_empty_coverage() {
    for (summary, diagnostic) in [
        (
            summary(100_000, 99_999, 3, 3),
            "line coverage is 99999/100000",
        ),
        (summary(10, 10, 3, 2), "function coverage is 2/3"),
        (summary(0, 0, 0, 0), "line coverage is 0/0"),
        (
            "{\"data\":[]}\n".into(),
            "expected one LLVM coverage summary",
        ),
        ("invalid JSON\n".into(), "expected value"),
    ] {
        let directory = setup();
        directory.write("summary.json", &summary);
        let output = command(&directory, &["--html"]).output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{output:?}"
        );
        assert!(
            directory
                .0
                .join("target/coverage/html/index.html")
                .is_file()
        );
        assert_eq!(steps(&calls(&directory)).last(), Some(&"export"));
    }
}

#[test]
fn cov_stops_when_a_tool_fails() {
    for step in [
        "metadata",
        "version",
        "sysroot",
        "build",
        "tests",
        "llvm-profdata",
        "report",
        "show",
        "export",
    ] {
        let directory = setup();
        let output = command(&directory, &["--html"])
            .env("XTASK_TEST_FAIL_STEP", step)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("failed:"));
        assert_eq!(steps(&calls(&directory)).last(), Some(&step));
    }
}

#[test]
fn cov_sets_up_missing_llvm_tools_and_checks_the_compiler_sysroot() {
    for (install, fail) in [(true, false), (false, false), (false, true)] {
        let directory = setup();
        fs::remove_file(
            directory
                .0
                .join("sysroot/lib/rustlib/test-host/bin/llvm-cov"),
        )
        .unwrap();
        let output = command(&directory, &[])
            .env("XTASK_TEST_INSTALL", if install { "yes" } else { "no" })
            .env("XTASK_TEST_FAIL_STEP", if fail { "rustup" } else { "" })
            .output()
            .unwrap();
        assert_eq!(output.status.success(), install, "{output:?}");
        assert_eq!(
            &calls(&directory)[3][5..],
            ["component", "add", "llvm-tools-preview"]
        );

        if !install && !fail {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("LLVM coverage tools are missing")
            );
        }
    }
}

#[test]
fn cov_rejects_missing_inputs_and_malformed_tool_output() {
    for (file, contents, diagnostic) in [
        ("metadata.json", "invalid\n", "expected value"),
        ("artifacts.json", "invalid\n", "expected value"),
        (
            "artifacts.json",
            "",
            "Cargo did not produce any workspace coverage executables",
        ),
    ] {
        let directory = setup();
        directory.write(file, contents);
        let output = command(&directory, &[]).output().unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{output:?}"
        );
    }

    for (variable, value, diagnostic) in [
        (
            "XTASK_TEST_VERSION",
            "no host",
            "rustc did not report its host target",
        ),
        (
            "XTASK_TEST_NO_PROFILES",
            "yes",
            "tests did not produce any coverage profiles",
        ),
    ] {
        let directory = setup();
        let output = command(&directory, &[])
            .env(variable, value)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{output:?}"
        );
    }

    let directory = setup();
    fs::remove_dir_all(directory.0.join("workspace with spaces/src")).unwrap();
    fs::create_dir(directory.0.join("workspace with spaces/src")).unwrap();
    let output = command(&directory, &[]).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no workspace sources found"));
}

#[test]
fn cov_help_and_invalid_arguments_do_not_run_tools() {
    let directory = setup();

    for args in [
        &["--help"][..],
        &["-h"],
        &["--unknown"],
        &["--html", "extra"],
    ] {
        let output = command(&directory, args).output().unwrap();
        assert_eq!(
            output.status.success(),
            args[0] == "--help" || args[0] == "-h"
        );
        assert!(!directory.0.join("calls").exists());
    }
}

#[test]
fn cov_finds_cargo_and_rustc_on_path() {
    let directory = setup();
    fs::remove_dir_all(directory.0.join("target/coverage")).unwrap();
    let output = command(&directory, &["--html"])
        .env_remove("CARGO")
        .env_remove("RUSTC")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
}
