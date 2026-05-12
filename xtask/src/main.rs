#![warn(clippy::pedantic)]

use std::{env, error, fs, io::Write, path, process, time};

#[cfg(unix)]
use std::os::unix::process::CommandExt as _;

use clap::{Parser, Subcommand};
use wait_timeout::ChildExt;

const TIMEOUT: time::Duration = time::Duration::from_secs(10);
const RUNNERS: &[(&str, &str)] = &[
    (
        "started_at_EL1",
        "qemu-system-aarch64 -machine xlnx-zcu102 -m 2G -nographic -no-reboot -semihosting-config enable=on,target=native -kernel",
    ),
    (
        "started_at_EL3",
        "qemu-system-aarch64 -machine xlnx-zcu102,secure=on,virtualization=on -m 2G -nographic -no-reboot -semihosting-config enable=on,target=native -kernel",
    ),
];
const EXAMPLE_FEATURES: &[(&str, &str)] = &[("newlib", "std")];
const EXAMPLE_CRATES: &[&str] = &["libtest"];

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run tests
    Test {
        /// Write the test results to a file in libtest JSON format
        #[arg(long, name = "FILE")]
        log_file: Option<path::PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();
    match args.command {
        Command::Test { log_file } => test(log_file),
    }
}

struct LogFile {
    file: Option<fs::File>,
}

impl LogFile {
    fn new(file_path: Option<path::PathBuf>) -> Result<Self, Box<dyn error::Error>> {
        Ok(Self {
            file: file_path.map(fs::File::create).transpose()?,
        })
    }

    fn start_test_suite(&mut self, test_count: usize) -> Result<(), Box<dyn error::Error>> {
        if let Some(f) = &mut self.file {
            writeln!(
                f,
                "{{ \"type\": \"suite\", \"event\": \"started\", \"test_count\": {test_count} }}",
            )?;
        }
        Ok(())
    }

    fn add_test_result(&mut self, name: &str, ok: bool) -> Result<(), Box<dyn error::Error>> {
        let event = if ok { "ok" } else { "failed" };
        if let Some(f) = &mut self.file {
            writeln!(
                f,
                "{{ \"type\": \"test\", \"event\": \"started\", \"name\": \"{name}\" }}",
            )?;
            writeln!(
                f,
                "{{ \"type\": \"test\", \"event\": \"{event}\", \"name\": \"{name}\" }}",
            )?;
        }
        Ok(())
    }

    fn end_test_suite(
        &mut self,
        num_passed: usize,
        num_failed: usize,
    ) -> Result<(), Box<dyn error::Error>> {
        let event = if num_failed == 0 { "ok" } else { "failed" };
        if let Some(f) = &mut self.file {
            writeln!(
                f,
                "{{ \"type\": \"suite\", \"event\": \"{event}\", \"passed\": {num_passed}, \"failed\": {num_failed}, \"ignored\": 0, \"measured\": 0, \"filtered_out\": 0 }}",
            )?;
        }
        Ok(())
    }
}

struct TestOutcome {
    passed: bool,
    reason: Option<String>,
    expected_stdout: Option<String>,
    captured_stdout: String,
}

fn discover_examples(cargo: &std::ffi::OsStr) -> Vec<String> {
    let output = process::Command::new(cargo)
        .arg("run")
        .arg("--example")
        .output()
        .expect("failed to execute cargo");
    let stderr = String::from_utf8(output.stderr).unwrap();
    stderr
        .split('\n')
        .filter(|l| l.starts_with(' ')) // Only the example names are indented
        .map(|l| l.trim().to_owned())
        .collect()
}

fn run_single_test(
    cargo: &std::ffi::OsStr,
    example: &str,
    features: &str,
    runner: &str,
) -> TestOutcome {
    let build_status = process::Command::new(cargo)
        .arg("build")
        .arg("--target")
        .arg("aarch64-unknown-none")
        .arg("--example")
        .arg(example)
        .arg("--features")
        .arg(features)
        .current_dir("zynqmp")
        .spawn()
        .expect("failed to spawn build")
        .wait()
        .unwrap();

    if !build_status.success() {
        let reason = build_status.code().map_or_else(
            || "terminated".to_string(),
            |code| format!("exit code {code}"),
        );
        return TestOutcome {
            passed: false,
            reason: Some(reason),
            expected_stdout: None,
            captured_stdout: String::new(),
        };
    }

    let stdout_path = env::temp_dir().join(format!("xtask-{}.stdout", process::id()));
    let mut child = {
        let mut cmd = process::Command::new(cargo);
        cmd.arg("run")
            .arg("--target")
            .arg("aarch64-unknown-none")
            .arg("--example")
            .arg(example)
            .arg("--features")
            .arg(features)
            .current_dir("zynqmp")
            .env("CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUNNER", runner)
            .stdin(process::Stdio::null())
            .stdout(fs::File::create(&stdout_path).expect("failed to create stdout capture file"));
        #[cfg(unix)]
        cmd.process_group(0);
        cmd.spawn().expect("failed to spawn example")
    };

    let (run_status, timed_out) = wait_and_kill(&mut child);
    let captured_stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    fs::remove_file(&stdout_path).ok();

    let stdout_path = path::Path::new("zynqmp/examples").join(format!("{example}.stdout"));
    let expected_stdout = stdout_path
        .exists()
        .then(|| fs::read_to_string(&stdout_path).expect("failed to read .stdout file"));
    let output_ok = expected_stdout
        .as_deref()
        .is_none_or(|expected| captured_stdout == expected);

    let passed = run_status.success() && output_ok;
    let reason = if passed {
        None
    } else if run_status.success() {
        Some("unexpected output".to_string())
    } else if let Some(code) = run_status.code() {
        Some(format!("exit code {code}"))
    } else {
        Some((if timed_out { "timeout" } else { "terminated" }).to_string())
    };

    TestOutcome {
        passed,
        reason,
        expected_stdout,
        captured_stdout,
    }
}

fn run_crate_test(
    cargo: &std::ffi::OsStr,
    crate_name: &str,
    crate_dir: &path::Path,
    runner: &str,
) -> TestOutcome {
    let stdout_path = env::temp_dir().join(format!("xtask-{}.stdout", process::id()));
    let mut child = {
        let mut cmd = process::Command::new(cargo);
        cmd.arg("test")
            .arg("--target")
            .arg("aarch64-unknown-none")
            .arg("--package")
            .arg(format!("zynqmp-{crate_name}"))
            .current_dir(crate_dir)
            .env("CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUNNER", runner)
            .stdin(process::Stdio::null())
            .stdout(fs::File::create(&stdout_path).expect("failed to create stdout capture file"));
        #[cfg(unix)]
        cmd.process_group(0);
        cmd.spawn().expect("failed to spawn test")
    };

    let (run_status, timed_out) = wait_and_kill(&mut child);
    let captured_stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
    fs::remove_file(&stdout_path).ok();

    let stdout_path = path::Path::new("examples").join(format!("{crate_name}.stdout"));
    let expected_stdout = fs::read_to_string(&stdout_path)
        .unwrap_or_else(|_| panic!("{} not found", stdout_path.display()));

    let normalized = normalize_output(&captured_stdout);
    let passed = normalized == expected_stdout;
    let reason = if passed {
        None
    } else if timed_out {
        Some("timeout".to_string())
    } else if let Some(code) = run_status.code() {
        Some(format!("exit code {code}"))
    } else {
        Some("terminated".to_string())
    };

    TestOutcome {
        passed,
        reason,
        expected_stdout: Some(expected_stdout),
        captured_stdout: normalized,
    }
}

fn test(log_file_path: Option<path::PathBuf>) -> Result<(), Box<dyn error::Error>> {
    let cargo = env::var_os("CARGO").unwrap();
    let examples = discover_examples(&cargo);

    println!("running {} examples", examples.len());

    let mut num_passed = 0;
    let mut num_failed = 0;

    let mut log_file = LogFile::new(log_file_path)?;
    log_file.start_test_suite((examples.len() + EXAMPLE_CRATES.len()) * RUNNERS.len())?;

    for example in &examples {
        let features = EXAMPLE_FEATURES
            .iter()
            .find(|(e, _)| e == example)
            .map_or("", |(_, f)| *f);

        for (variant, runner) in RUNNERS {
            print!("test {example} ({variant}) ... ");
            let outcome = run_single_test(&cargo, example, features, runner);
            record_outcome(
                &outcome,
                &format!("[example] {example}#{variant}"),
                &mut num_passed,
                &mut num_failed,
                &mut log_file,
            )?;
        }
    }

    for crate_name in EXAMPLE_CRATES {
        let crate_dir = path::Path::new("examples").join(crate_name);

        for (variant, runner) in RUNNERS {
            print!("test {crate_name} ({variant}) ... ");
            let outcome = run_crate_test(&cargo, crate_name, &crate_dir, runner);
            record_outcome(
                &outcome,
                &format!("[example-crate] {crate_name}#{variant}"),
                &mut num_passed,
                &mut num_failed,
                &mut log_file,
            )?;
        }
    }

    println!(
        "\ntest result: {}. {num_passed} passed; {num_failed} failed",
        if num_failed > 0 { "FAILED" } else { "ok" }
    );

    log_file.end_test_suite(num_passed, num_failed)?;

    if num_failed > 0 {
        println!("\nerror: test failed");
        process::exit(1);
    }

    Ok(())
}

fn wait_and_kill(child: &mut process::Child) -> (process::ExitStatus, bool) {
    if let Some(status) = child.wait_timeout(TIMEOUT).unwrap() {
        (status, false)
    } else {
        #[cfg(unix)]
        unsafe {
            libc::kill(-child.id().cast_signed(), libc::SIGKILL);
        }
        // On non-Unix platforms, only the direct child process is killed.
        // Grandchildren (e.g. QEMU spawned by the runner) may keep running.
        // The Unix fix (killing the process group) has no direct equivalent on
        // Windows. The proper solution there is a Job Object with
        // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, which requires unsafe Windows
        // API calls and the `windows-sys` dependency.
        #[cfg(not(unix))]
        child.kill().expect("failed to kill child");
        (child.wait().expect("failed to wait for child"), true)
    }
}

fn record_outcome(
    outcome: &TestOutcome,
    log_name: &str,
    num_passed: &mut usize,
    num_failed: &mut usize,
    log_file: &mut LogFile,
) -> Result<(), Box<dyn error::Error>> {
    if outcome.passed {
        println!("ok");
        *num_passed += 1;
        if outcome.expected_stdout.is_none() {
            print!("{}", outcome.captured_stdout);
        }
    } else {
        println!("FAILED ({})", outcome.reason.as_deref().unwrap_or(""));
        *num_failed += 1;
        if let Some(expected) = &outcome.expected_stdout {
            let diff = similar::TextDiff::from_lines(expected.as_str(), &outcome.captured_stdout);
            print!("{}", diff.unified_diff().header("expected", "actual"));
        } else {
            print!("{}", outcome.captured_stdout);
        }
    }
    log_file.add_test_result(log_name, outcome.passed)
}

fn normalize_output(output: &str) -> String {
    let normalized = output
        .lines()
        .map(|line| match line.find("; finished in ") {
            Some(pos) => &line[..pos],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    if output.ends_with('\n') {
        normalized.trim_end_matches('\n').to_string() + "\n"
    } else {
        normalized
    }
}
