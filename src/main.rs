use std::path::PathBuf;
use trekr::app::{App, UiCaptureOptions};
use trekr::state;

enum RunMode {
    Interactive,
    Capture(UiCaptureOptions),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMode {
    Persisted,
    Demo,
    Empty,
}

struct LaunchOptions {
    run_mode: RunMode,
    state_mode: StateMode,
    state_file: PathBuf,
}

fn parse_state_mode(value: &str) -> Result<StateMode, String> {
    match value {
        "persisted" => Ok(StateMode::Persisted),
        "demo" => Ok(StateMode::Demo),
        "empty" => Ok(StateMode::Empty),
        other => Err(format!("unknown state mode: {other}")),
    }
}

fn parse_launch_options() -> Result<LaunchOptions, String> {
    let mut args = std::env::args().skip(1);
    let mut capture_dir = None;
    let mut state_mode = StateMode::Persisted;
    let mut state_file = PathBuf::from("artifacts/state/last-run.json");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--capture-ui" => {
                if capture_dir.is_none() {
                    capture_dir = Some(PathBuf::from("artifacts/screenshots"));
                }
            }
            "--capture-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--capture-dir requires a path".to_owned())?;
                capture_dir = Some(PathBuf::from(value));
            }
            "--state-mode" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--state-mode requires persisted|demo|empty".to_owned())?;
                state_mode = parse_state_mode(&value)?;
            }
            "--state-file" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--state-file requires a path".to_owned())?;
                state_file = PathBuf::from(value);
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    let run_mode = match capture_dir {
        Some(output_dir) => RunMode::Capture(UiCaptureOptions { output_dir }),
        None => RunMode::Interactive,
    };

    Ok(LaunchOptions {
        run_mode,
        state_mode,
        state_file,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = parse_launch_options().map_err(|err| format!("argument error: {err}"))?;
    let mut app = match options.state_mode {
        StateMode::Persisted => {
            if options.state_file.exists() {
                match state::load(&options.state_file) {
                    Ok(state) => App::from_persisted_state(state),
                    Err(_) => App::new_demo(),
                }
            } else {
                App::new_demo()
            }
        }
        StateMode::Demo => App::new_demo(),
        StateMode::Empty => App::new_empty(),
    };
    println!("{}", app.bootstrap_summary());
    match options.run_mode {
        RunMode::Interactive => {
            let result = app.run();
            if result.is_ok() && options.state_mode == StateMode::Persisted {
                state::save(&options.state_file, &app.persisted_state())?;
            }
            result
        }
        RunMode::Capture(capture) => app.capture_ui_pages(capture),
    }
}
