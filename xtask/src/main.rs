use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitCode};

const ABLETON_LINK_HEADER: &str = "vendor/ableton-link/include/ableton/Link.hpp";

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(ExitCode::SUCCESS);
    };

    match command.as_str() {
        "setup" => {
            ensure_submodule()?;
            Ok(ExitCode::SUCCESS)
        }
        "run" => {
            ensure_submodule()?;
            spawn_cargo(["run"], args)
        }
        "run-demo" => {
            ensure_submodule()?;
            spawn_cargo(["run", "--", "--state-mode", "demo"], args)
        }
        "run-empty" => {
            ensure_submodule()?;
            spawn_cargo(["run", "--", "--state-mode", "empty"], args)
        }
        "check" => {
            ensure_submodule()?;
            spawn_cargo(["check"], args)
        }
        "fmt" => spawn_cargo(["fmt"], args),
        "capture-ui" => {
            ensure_submodule()?;
            spawn_powershell_script(
                ["scripts/capture-ui-screens.ps1", "-StateMode", "demo"],
                args,
            )
        }
        "review-ui" => {
            ensure_submodule()?;
            spawn_powershell_script(
                ["scripts/review-ui-screens.ps1", "-StateMode", "demo"],
                args,
            )
        }
        "ui-review" => {
            ensure_submodule()?;
            spawn_powershell_script(["scripts/run-ui-review.ps1", "-StateMode", "demo"], args)
        }
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(ExitCode::SUCCESS)
        }
        other => Err(format!("unknown xtask command: {other}")),
    }
}

fn ensure_submodule() -> Result<(), String> {
    if Path::new(ABLETON_LINK_HEADER).exists() {
        return Ok(());
    }

    let status = Command::new("git")
        .args([
            "submodule",
            "update",
            "--init",
            "--recursive",
            "vendor/ableton-link",
        ])
        .status()
        .map_err(|error| format!("failed to start git submodule update: {error}"))?;

    if !status.success() {
        return Err(format!("git submodule update failed with status {status}"));
    }

    if !Path::new(ABLETON_LINK_HEADER).exists() {
        return Err(format!(
            "submodule initialized but missing expected header at {ABLETON_LINK_HEADER}"
        ));
    }

    Ok(())
}

fn spawn_cargo<const N: usize, I>(base_args: [&str; N], extra_args: I) -> Result<ExitCode, String>
where
    I: IntoIterator<Item = String>,
{
    let mut command = Command::new("cargo");
    for arg in base_args {
        command.arg(arg);
    }
    for arg in extra_args {
        command.arg(arg);
    }
    run_command(command)
}

fn spawn_powershell_script<const N: usize, I>(
    base_args: [&str; N],
    extra_args: I,
) -> Result<ExitCode, String>
where
    I: IntoIterator<Item = String>,
{
    let mut command = Command::new("powershell");
    command.args(["-ExecutionPolicy", "Bypass", "-File"]);
    for arg in base_args {
        command.arg(arg);
    }
    for arg in extra_args {
        command.arg(arg);
    }
    run_command(command)
}

fn run_command(mut command: Command) -> Result<ExitCode, String> {
    let status = command
        .status()
        .map_err(|error| format!("failed to start command: {error}"))?;

    Ok(match status.code() {
        Some(code) => ExitCode::from(code as u8),
        None => ExitCode::FAILURE,
    })
}

fn print_usage() {
    let lines: [OsString; 10] = [
        "usage: cargo xtask <command> [extra args]".into(),
        "".into(),
        "commands:".into(),
        "  setup       initialize vendor/ableton-link".into(),
        "  run         initialize submodule if needed, then cargo run".into(),
        "  run-demo    initialize submodule if needed, then cargo run -- --state-mode demo".into(),
        "  run-empty   initialize submodule if needed, then cargo run -- --state-mode empty".into(),
        "  check       initialize submodule if needed, then cargo check".into(),
        "  capture-ui  initialize submodule if needed, then run screenshot capture".into(),
        "  ui-review   initialize submodule if needed, then run screenshot capture + review".into(),
    ];

    for line in lines {
        println!("{}", line.to_string_lossy());
    }
}
