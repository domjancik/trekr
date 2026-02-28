use std::path::PathBuf;
use trekr::app::{App, UiCaptureOptions};

enum RunMode {
    Interactive,
    Capture(UiCaptureOptions),
}

fn parse_run_mode() -> Result<RunMode, String> {
    let mut args = std::env::args().skip(1);
    let mut capture_dir = None;

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
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    Ok(match capture_dir {
        Some(output_dir) => RunMode::Capture(UiCaptureOptions { output_dir }),
        None => RunMode::Interactive,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let run_mode = parse_run_mode().map_err(|err| format!("argument error: {err}"))?;
    let mut app = App::new();
    println!("{}", app.bootstrap_summary());
    match run_mode {
        RunMode::Interactive => app.run(),
        RunMode::Capture(options) => app.capture_ui_pages(options),
    }
}
