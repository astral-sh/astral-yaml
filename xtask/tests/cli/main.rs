use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

#[cfg(unix)]
mod fuzz;

struct Directory(PathBuf);

impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "astral-yaml-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    #[cfg(unix)]
    fn write(&self, name: &str, content: &str) {
        fs::write(self.0.join(name), content).unwrap();
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
        command.args(args).current_dir(&self.0);

        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn unknown_commands_fail() {
    let directory = Directory::new();

    for command in ["events", "conformance", "unknown"] {
        let output = directory.run(&[command]);
        assert_eq!(output.status.code(), Some(1), "{command}");
        assert!(output.stdout.is_empty(), "{command}");
        assert!(
            String::from_utf8_lossy(&output.stderr).starts_with("usage: cargo xtask fuzz"),
            "{command}"
        );
    }
}

#[test]
fn help_works_outside_the_repository_root() {
    let directory = Directory::new();

    for args in [vec![], vec!["-h"], vec!["--help"]] {
        let output = directory.run(&args);
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "usage: cargo xtask fuzz [cargo-fuzz arguments...]\n\nWithout arguments, fuzz runs parse, serde, decode, and typed for 60 seconds each.\n"
        );
    }
}

#[test]
fn fuzz_requires_uv() {
    let directory = Directory::new();
    let output = directory
        .command(&["fuzz"])
        .env("PATH", &directory.0)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("failed to run uv"));
}
