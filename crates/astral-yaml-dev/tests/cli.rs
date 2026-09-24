use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

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
    fn write(&self, name: &str, content: &str) {
        fs::write(self.0.join(name), content).unwrap();
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_astral-yaml-dev"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn events_and_command_errors() {
    let directory = Directory::new();
    directory.write("valid.yaml", "[a, b]");
    let output = directory.run(&["events", "valid.yaml"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "+STR\n+DOC\n+SEQ []\n=VAL :a\n=VAL :b\n-SEQ\n-DOC\n-STR\n"
    );
    directory.write("invalid.yaml", "[");
    for args in [
        vec!["events"],
        vec!["events", "missing"],
        vec!["events", "invalid.yaml"],
        vec!["unknown"],
        vec!["conformance", "missing"],
        vec!["conformance", "."],
        vec![],
    ] {
        let output = directory.run(&args);
        assert!(!output.status.success(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}

#[test]
fn conformance_reports_rejections_acceptance_and_event_differences() {
    let directory = Directory::new();
    directory.write("in.yaml", "[");
    let output = directory.run(&["conformance", "."]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("FAIL"));
    directory.write("in.yaml", "abc");
    directory.write("error", "");
    let output = directory.run(&["conformance", "."]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("accepted malformed input"));
    fs::remove_file(directory.0.join("error")).unwrap();
    for expected in [
        "wrong\n",
        "+STR\n",
        "+STR\n+DOC\n=VAL :abc\n-DOC\n-STR\nextra\n",
    ] {
        directory.write("test.event", expected);
        let output = directory.run(&["conformance", "."]);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("event"));
    }
    fs::remove_file(directory.0.join("test.event")).unwrap();
    assert!(!directory.run(&["conformance", "."]).status.success());
}
