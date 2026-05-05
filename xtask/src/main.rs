#![warn(clippy::pedantic)]

use std::{
    env, error, fs,
    io::{Read, Write},
    path, process, thread, time,
};

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
    log_file.start_test_suite(examples.len() * RUNNERS.len())?;

    for example in examples {
        let features = EXAMPLE_FEATURES
            .iter()
            .find(|(e, _)| *e == example)
            .map_or("", |(_, f)| *f);

        for (variant, runner) in RUNNERS {
            print!("test {example} ({variant}) ... ");

            let mut status = process::Command::new(&cargo)
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

            let mut timeout = false;
            let mut captured_stdout = String::new();

            if status.success() {
                let mut child = process::Command::new(&cargo)
                    .arg("run")
                    .arg("--target")
                    .arg("aarch64-unknown-none")
                    .arg("--example")
                    .arg(example)
                    .arg("--features")
                    .arg(features)
                    .current_dir("zynqmp")
                    .env("CARGO_TARGET_AARCH64_UNKNOWN_NONE_RUNNER", runner)
                    .stdout(process::Stdio::piped())
                    .spawn()
                    .expect("failed to spawn example");

                let mut child_stdout = child.stdout.take().expect("failed to take stdout");
                let reader = thread::spawn(move || {
                    let mut buf = String::new();
                    child_stdout.read_to_string(&mut buf).map(|_| buf)
                });

                status = if let Some(status) = child.wait_timeout(TIMEOUT).unwrap() {
                    status
                } else {
                    timeout = true;
                    child.kill().expect("failed to kill example");
                    child.wait().expect("failed to wait for example")
                };

                captured_stdout = reader.join().unwrap().unwrap_or_default();
            }

            let stdout_path = path::Path::new("zynqmp/examples").join(format!("{example}.stdout"));
            let expected_stdout = stdout_path
                .exists()
                .then(|| fs::read_to_string(&stdout_path).expect("failed to read .stdout file"));
            let output_ok = expected_stdout
                .as_deref()
                .is_none_or(|e| captured_stdout == e);

            if status.success() && output_ok {
                println!("ok");
                num_passed += 1;
            } else {
                let reason = if status.success() {
                    "unexpected output".to_string()
                } else if let Some(code) = status.code() {
                    format!("exit code {code}")
                } else {
                    (if timeout { "timeout" } else { "terminated" }).to_string()
                };
                println!("FAILED ({reason})");
                num_failed += 1;
            }
            if !output_ok {
                println!("--- expected stdout ---");
                print!("{}", expected_stdout.as_deref().unwrap_or(""));
                println!("--- actual stdout ---");
            }
            print!("{captured_stdout}");

            log_file.add_test_result(
                &format!("[example] {example}#{variant}"),
                status.success() && output_ok,
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
