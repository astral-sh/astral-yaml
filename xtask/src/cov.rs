use std::collections::BTreeSet;
use std::error::Error;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::{env, fs};

use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const PACKAGES: [&str; 3] = ["astral-yaml", "astral-yaml-serde", "astral-yaml-python"];

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

#[derive(Default, Deserialize)]
struct Totals {
    lines: Count,
    functions: Count,
}

#[derive(Default, Deserialize)]
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
            collect_sources(&path, sources)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
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
            && members.contains(&package_id)
        {
            executables.insert(path);
        }
    }

    if executables.is_empty() {
        return Err("Cargo did not produce any workspace coverage executables".into());
    }

    Ok(executables)
}

fn enforce_thresholds(totals: &Totals) -> Result<()> {
    for (name, total) in [("line", &totals.lines), ("function", &totals.functions)] {
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

fn binding_totals(lcov: &str) -> Result<Totals> {
    // Rust unit tests and the extension contain separate copies of the binding.
    // LLVM's LCOV line records combine their hits. Its per-function line totals
    // use only the most-covered copy of each function.
    let mut totals = Totals::default();
    for record in lcov.lines() {
        if let Some(line) = record.strip_prefix("DA:") {
            let (_, hits) = line.split_once(',').ok_or("invalid LCOV line record")?;
            totals.lines.count += 1;
            totals.lines.covered += u64::from(hits.parse::<u64>()? > 0);
        } else if let Some(count) = record.strip_prefix("FNF:") {
            totals.functions.count += count.parse::<u64>()?;
        } else if let Some(count) = record.strip_prefix("FNH:") {
            totals.functions.covered += count.parse::<u64>()?;
        }
    }
    Ok(totals)
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

fn python_coverage(root: &Path, directory: &Path, profile: &Path) -> Result<(PathBuf, PathBuf)> {
    let mut uv = Command::new("uv");
    uv.current_dir(root)
        .env("UV_PROJECT_ENVIRONMENT", directory.join("venv"))
        .env("CARGO_TARGET_DIR", directory.join("build"))
        .env("LLVM_PROFILE_FILE", profile)
        .args(["run", "--locked", "--no-dev", "--group", "test"])
        .args([
            "--config-settings-package",
            "astral-yaml:maturin.build-args=--profile dev --locked -- -Cinstrument-coverage",
        ]);
    if let Some(python) = env::var_os("PYO3_PYTHON") {
        uv.arg("--python").arg(python);
    }
    let paths = output(uv.args([
        "python",
        "-c",
        "import json, sys; from astral_yaml import astral_yaml; print(json.dumps([sys.executable, astral_yaml.__file__]))",
    ]))?;
    let (python, extension): (PathBuf, PathBuf) = serde_json::from_str(&paths)?;
    for args in [&["run", "-m", "pytest"][..], &["report"]] {
        execute(
            Command::new(&python)
                .current_dir(root)
                .env("LLVM_PROFILE_FILE", profile)
                .env("COVERAGE_FILE", directory.join(".coverage"))
                .args(["-m", "coverage"])
                .args(args),
        )?;
    }
    Ok((python, extension))
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
    let mut binding_sources = Vec::new();
    let mut members = BTreeSet::new();

    for package in &metadata.packages {
        if PACKAGES.contains(&package.name.as_str()) {
            let files = if package.name == "astral-yaml-python" {
                &mut binding_sources
            } else {
                &mut sources
            };
            collect_sources(&package.manifest_path.with_file_name("src"), files)?;
            members.insert(package.id.clone());
        }
    }

    sources.sort();
    binding_sources.sort();

    if sources.is_empty() || binding_sources.is_empty() {
        return Err("no YAML sources found for coverage".into());
    }

    let profile_pattern = profiles.join("%m-%p.profraw");
    let (interpreter, extension) = python_coverage(&root, &directory, &profile_pattern)?;

    // Build separately so test stdout cannot be mistaken for Cargo JSON messages.
    // The explicit host target keeps build scripts and proc macros uninstrumented.
    let test_command = || {
        let mut tests = Command::new(&cargo);
        tests
            .current_dir(&root)
            .args(["test", "--tests", "--all-features", "--locked", "--target"])
            .arg(&toolchain.host)
            .arg("--target-dir")
            .arg(directory.join("build"))
            .env("CARGO_ENCODED_RUSTFLAGS", coverage_flags())
            .env("LLVM_PROFILE_FILE", &profile_pattern)
            .env("PYO3_PYTHON", &interpreter)
            .env("CARGO_INCREMENTAL", "0");

        for package in PACKAGES {
            tests.args(["--package", package]);
        }

        tests
    };
    let messages =
        output(test_command().args(["--no-run", "--message-format=json-render-diagnostics"]))?;
    let mut executables = artifacts(&messages, &members)?;
    executables.insert(extension);
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
        &sources,
        &binding_sources,
        html,
    )
}

fn report(
    llvm_cov: &Path,
    directory: &Path,
    profile: PathBuf,
    executables: BTreeSet<PathBuf>,
    sources: &[PathBuf],
    binding_sources: &[PathBuf],
    html: bool,
) -> Result<()> {
    let mut report_args = vec![OsString::from("-instr-profile"), profile.into_os_string()];

    for executable in executables {
        report_args.push("-object".into());
        report_args.push(executable.into_os_string());
    }

    report_args.push("-sources".into());
    execute(
        Command::new(llvm_cov)
            .arg("report")
            .args(&report_args)
            .args(sources),
    )?;

    let binding_lcov = output(
        Command::new(llvm_cov)
            .args([
                "export",
                "-format=lcov",
                "-skip-functions",
                "-skip-branches",
            ])
            .args(&report_args)
            .args(binding_sources),
    )?;
    let binding = binding_totals(&binding_lcov)?;
    println!(
        "Python binding: {}/{} lines and {}/{} functions covered",
        binding.lines.covered,
        binding.lines.count,
        binding.functions.covered,
        binding.functions.count
    );

    if html {
        let html_directory = directory.join("html");
        reset_directory(&html_directory)?;
        execute(
            Command::new(llvm_cov)
                .args(["show", "-format=html", "-show-line-counts-or-regions"])
                .arg("-output-dir")
                .arg(&html_directory)
                .args(&report_args)
                .args(sources),
        )?;
        let binding_html = html_directory.join("python.html");
        execute(
            Command::new(llvm_cov)
                .args([
                    "show",
                    "-format=html",
                    "-show-instantiations=false",
                    "-show-line-counts",
                ])
                .args(&report_args)
                .args(binding_sources)
                .stdout(fs::File::create(&binding_html)?),
        )?;
        println!(
            "HTML report: {}",
            html_directory.join("index.html").display()
        );
        println!("Python binding HTML report: {}", binding_html.display());
    }

    let summary = output(
        Command::new(llvm_cov)
            .args(["export", "-summary-only"])
            .args(&report_args)
            .args(sources),
    )?;
    fs::write(directory.join("summary.json"), &summary)?;
    let report: Report = serde_json::from_str(&summary)?;
    let [coverage] = report.data.as_slice() else {
        return Err("expected one LLVM coverage summary".into());
    };
    enforce_thresholds(&coverage.totals)?;
    enforce_thresholds(&binding)
}
