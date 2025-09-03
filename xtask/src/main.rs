#![warn(clippy::pedantic)]

use std::{env, error, fs, io::Write, path, process, time};

use clap::{Parser, Subcommand};
use wait_timeout::ChildExt;

const TIMEOUT: time::Duration = time::Duration::from_secs(10);
const RUNNERS: [(&str, &str); 2] = [
    (
        "started_at_EL1",
        "qemu-system-aarch64 -machine xlnx-zcu102 -m 2G -nographic -semihosting-config enable=on,target=native -kernel",
    ),
    (
        "started_at_EL3",
        "qemu-system-aarch64 -machine xlnx-zcu102,secure=on,virtualization=on -m 2G -nographic -semihosting-config enable=on,target=native -kernel",
    ),
];

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

fn test(log_file_path: Option<path::PathBuf>) -> Result<(), Box<dyn error::Error>> {
    let cargo = env::var_os("CARGO").unwrap();
    let output = process::Command::new(&cargo)
        .arg("run")
        .arg("--example")
        .output()
        .expect("failed to execute cargo");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let examples = stderr
        .split('\n')
        .filter(|l| l.starts_with(' ')) // Only the example names are indented
        .map(str::trim)
        .collect::<Vec<_>>();

    println!("running {} examples", examples.len());

    let mut num_passed = 0;
    let mut num_failed = 0;

    let mut log_file = LogFile::new(log_file_path)?;
    log_file.start_test_suite(examples.len())?;

    for example in examples {
        for (variant, runner) in RUNNERS {
            print!("test {example} ({variant}) ... ");

            let mut child = process::Command::new(&cargo)
                .arg("run")
                .arg("--target")
                .arg("aarch64-unknown-none")
                .arg("--example")
                .arg(example)
                .current_dir("zynqmp")
                .env("CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUNNER", runner)
                .spawn()
                .expect("failed to spawn example");

            let mut timeout = false;
            let status = if let Some(status) = child.wait_timeout(TIMEOUT).unwrap() {
                status
            } else {
                timeout = true;
                child.kill().expect("failed to kill example");
                child.wait().expect("failed to wait for example")
            };

            if status.success() {
                println!("ok");
                num_passed += 1;
            } else {
                let reason = if let Some(code) = status.code() {
                    format!("exit code {code}")
                } else {
                    (if timeout { "timeout" } else { "terminated" }).to_string()
                };
                println!("FAILED ({reason})");
                num_failed += 1;
            }

            log_file
                .add_test_result(&format!("[example] {example}#{variant}"), status.success())?;
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
