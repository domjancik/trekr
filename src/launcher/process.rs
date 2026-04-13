use crate::launcher::cli::RunLauncherOptions;
use crate::launcher::models::InstalledBuild;
use std::process::Command;

pub fn run_installed(
    install: &InstalledBuild,
    options: &RunLauncherOptions,
) -> Result<i32, Box<dyn std::error::Error>> {
    if !install.binary_path.exists() {
        return Err(format!(
            "installed binary for branch '{}' was not found at {}",
            install.branch,
            install.binary_path.display()
        )
        .into());
    }

    let mut args = vec!["run".to_string()];
    if let Some(state_mode) = &options.state_mode {
        args.push("--state-mode".to_string());
        args.push(state_mode.clone());
    }
    if let Some(project) = &options.project {
        args.push("--state-file".to_string());
        args.push(project.display().to_string());
    }
    if let Some(window_mode) = &options.window_mode {
        args.push("--video-mode".to_string());
        args.push(window_mode.clone());
    }
    if let Some(ui_scale) = options.ui_scale {
        args.push("--ui-scale".to_string());
        args.push(ui_scale.to_string());
    }
    args.extend(options.extra_args.clone());

    let status = Command::new(&install.binary_path).args(args).status()?;
    Ok(status.code().unwrap_or(1))
}
