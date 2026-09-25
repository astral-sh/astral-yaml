use std::{
    collections::BTreeSet,
    env,
    error::Error,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PACKAGES: [&str; 2] = ["astral-yaml", "astral-yaml-serde"];

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    target_directory: PathBuf,
}

#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: PathBuf,
}

#[derive(Deserialize)]
#[serde(tag = "reason", rename_all = "kebab-case")]
enum Message {
    CompilerArtifact {
        package_id: String,
        executable: Option<PathBuf>,
    },
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct Report {
    data: Vec<Coverage>,
}

#[derive(Deserialize)]
struct Coverage {
    totals: Totals,
}

#[derive(Deserialize)]
struct Totals {
    lines: Count,
    functions: Count,
}

#[derive(Deserialize)]
struct Count {
    count: u64,
    covered: u64,
}

fn check_status(command: &Command, status: ExitStatus) -> Result<()> {
    if !status.success() {
        return Err(format!(
            "{} failed: {status}",
            command.get_program().to_string_lossy()
        )
        .into());
    }

    Ok(())
}

fn execute(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    check_status(command, status)
}

fn output(command: &mut Command) -> Result<String> {
    let output = command.stderr(Stdio::inherit()).output()?;
    check_status(command, output.status)?;

    Ok(String::from_utf8(output.stdout)?)
}

fn collect_sources(directory: &Path, sources: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();

        if path.is_dir() {
            if path.file_name().is_some_and(|name| name != "tests") {
                collect_sources(&path, sources)?;
            }
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_some_and(|name| name != "tests.rs")
        {
            sources.push(path);
        }
    }

    Ok(())
}

fn reset_directory(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }

    fs::create_dir_all(path)?;
    Ok(())
}

fn coverage_flags() -> OsString {
    let mut flags = env::var_os("CARGO_ENCODED_RUSTFLAGS").unwrap_or_else(|| {
        env::var("RUSTFLAGS")
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("\x1f")
            .into()
    });

    if !flags.is_empty() {
        flags.push("\x1f");
    }

    flags.push("-C\x1finstrument-coverage");
    flags
}

fn artifacts(messages: &str, members: &BTreeSet<String>) -> Result<BTreeSet<PathBuf>> {
    let mut executables = BTreeSet::new();

    for line in messages.lines() {
        if let Message::CompilerArtifact {
            package_id,
            executable: Some(path),
        } = serde_json::from_str(line)?
        {
            if members.contains(&package_id) {
                executables.insert(path);
            }
        }
    }

    if executables.is_empty() {
        return Err("Cargo did not produce any workspace coverage executables".into());
    }

    Ok(executables)
}

fn enforce_thresholds(json: &str) -> Result<()> {
    let report: Report = serde_json::from_str(json)?;
    let [coverage] = report.data.as_slice() else {
        return Err("expected one LLVM coverage summary".into());
    };

    for (name, total) in [
        ("line", &coverage.totals.lines),
        ("function", &coverage.totals.functions),
    ] {
        // Compare counts, since displayed percentages can round up to 100%.
        if total.count == 0 || total.covered != total.count {
            return Err(format!(
                "{name} coverage is {}/{}; expected 100% with nonempty coverage",
                total.covered, total.count
            )
            .into());
        }
    }

    Ok(())
}

struct Toolchain {
    host: String,
    cov: PathBuf,
    profdata: PathBuf,
}

fn toolchain(root: &Path) -> Result<Toolchain> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let version = output(Command::new(&rustc).current_dir(root).arg("-vV"))?;
    let host = version
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc did not report its host target")?;
    let sysroot = output(
        Command::new(&rustc)
            .current_dir(root)
            .args(["--print", "sysroot"]),
    )?;
    let llvm = Path::new(sysroot.trim())
        .join("lib/rustlib")
        .join(host)
        .join("bin");
    let llvm_cov = llvm.join(format!("llvm-cov{}", env::consts::EXE_SUFFIX));
    let llvm_profdata = llvm.join(format!("llvm-profdata{}", env::consts::EXE_SUFFIX));

    if !llvm_cov.is_file() || !llvm_profdata.is_file() {
        execute(Command::new("rustup").current_dir(root).args([
            "component",
            "add",
            "llvm-tools-preview",
        ]))?;

        if !llvm_cov.is_file() || !llvm_profdata.is_file() {
            return Err(format!("LLVM coverage tools are missing from {}", llvm.display()).into());
        }
    }

    Ok(Toolchain {
        host: host.to_owned(),
        cov: llvm_cov,
        profdata: llvm_profdata,
    })
}

pub(crate) fn run(html: bool) -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let metadata: Metadata =
        serde_json::from_str(&output(Command::new(&cargo).current_dir(&root).args([
            "metadata",
            "--format-version=1",
            "--no-deps",
            "--locked",
        ]))?)?;
    let toolchain = toolchain(&root)?;
    let directory = metadata.target_directory.join("coverage");
    let profiles = directory.join("profiles");
    reset_directory(&profiles)?;

    let mut sources = Vec::new();
    let mut members = BTreeSet::new();

    for package in &metadata.packages {
        if PACKAGES.contains(&package.name.as_str()) {
            collect_sources(&package.manifest_path.with_file_name("src"), &mut sources)?;
            members.insert(package.id.clone());
        }
    }

    sources.sort();

    if sources.is_empty() {
        return Err("no YAML sources found for coverage".into());
    }

    // Build separately so test stdout cannot be mistaken for Cargo JSON messages.
    // The explicit host target keeps build scripts and proc macros uninstrumented.
    let test_command = || {
        let mut tests = Command::new(&cargo);
        tests
            .current_dir(&root)
            .args(["test", "--tests", "--locked", "--target"])
            .arg(&toolchain.host)
            .arg("--target-dir")
            .arg(directory.join("build"))
            .env("CARGO_ENCODED_RUSTFLAGS", coverage_flags())
            .env("LLVM_PROFILE_FILE", profiles.join("%m-%p.profraw"))
            .env("CARGO_INCREMENTAL", "0");

        for package in PACKAGES {
            tests.args(["--package", package]);
        }

        tests
    };
    let messages =
        output(test_command().args(["--no-run", "--message-format=json-render-diagnostics"]))?;
    let executables = artifacts(&messages, &members)?;
    execute(&mut test_command())?;

    let profile = directory.join("coverage.profdata");
    let mut merge = Command::new(toolchain.profdata);
    merge.args(["merge", "-sparse", "-o"]).arg(&profile);
    let mut raw_profiles = Vec::new();

    for entry in fs::read_dir(&profiles)? {
        let path = entry?.path();

        if path.extension().is_some_and(|ext| ext == "profraw") {
            raw_profiles.push(path);
        }
    }

    if raw_profiles.is_empty() {
        return Err("tests did not produce any coverage profiles".into());
    }

    raw_profiles.sort();
    execute(merge.args(raw_profiles))?;

    report(
        &toolchain.cov,
        &directory,
        profile,
        executables,
        sources,
        html,
    )
}

fn report(
    llvm_cov: &Path,
    directory: &Path,
    profile: PathBuf,
    executables: BTreeSet<PathBuf>,
    sources: Vec<PathBuf>,
    html: bool,
) -> Result<()> {
    let mut report_args = vec![OsString::from("-instr-profile"), profile.into_os_string()];

    for executable in executables {
        report_args.push("-object".into());
        report_args.push(executable.into_os_string());
    }

    report_args.push("-sources".into());
    report_args.extend(sources.into_iter().map(PathBuf::into_os_string));
    execute(Command::new(llvm_cov).arg("report").args(&report_args))?;

    if html {
        let html_directory = directory.join("html");
        reset_directory(&html_directory)?;
        execute(
            Command::new(llvm_cov)
                .args(["show", "-format=html", "-show-line-counts-or-regions"])
                .arg("-output-dir")
                .arg(&html_directory)
                .args(&report_args),
        )?;
        println!(
            "HTML report: {}",
            html_directory.join("index.html").display()
        );
    }

    let summary = output(
        Command::new(llvm_cov)
            .args(["export", "-summary-only"])
            .args(&report_args),
    )?;
    fs::write(directory.join("summary.json"), &summary)?;
    enforce_thresholds(&summary)
}
