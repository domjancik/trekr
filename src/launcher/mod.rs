pub mod catalog;
pub mod cli;
pub mod installs;
pub mod models;
pub mod process;
pub mod state;

use crate::launcher::cli::{LauncherCommand, RunLauncherOptions};
use crate::launcher::models::InstalledBuild;
use std::path::PathBuf;

const DEFAULT_REPO_URL: &str = "https://github.com/domjancik/trekr.git";

pub fn run_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let command = cli::parse_command_from(std::env::args().skip(1))?;
    execute(command)
}

pub fn execute(command: LauncherCommand) -> Result<(), Box<dyn std::error::Error>> {
    let state_path = state::default_state_path();
    let mut launcher_state = state::load(&state_path).unwrap_or_default();

    match command {
        LauncherCommand::Help => {
            cli::print_help(&mut std::io::stdout())?;
        }
        LauncherCommand::ListBranches { repo_url } => {
            let repo_url = resolve_repo_url(repo_url, &launcher_state);
            let branches = catalog::list_remote_branches(&repo_url)?;
            println!("Remote branches ({repo_url}):");
            for branch in branches {
                println!("  {branch}");
            }
        }
        LauncherCommand::Install {
            branch,
            repo_url,
            rebuild,
        } => {
            let repo_url = resolve_repo_url(repo_url, &launcher_state);
            let install = installs::install_branch(&repo_url, &branch, rebuild)?;
            upsert_install(&mut launcher_state.installs, install.clone());
            launcher_state.repo_url = Some(repo_url);
            launcher_state.last_selected_branch = Some(branch);
            state::save(&state_path, &launcher_state)?;
            print_install_summary(&install);
        }
        LauncherCommand::Run(options) => {
            let install = launcher_state
                .installs
                .iter()
                .find(|entry| entry.branch == options.branch)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "branch '{}' is not installed yet. Run `trekr-launcher install --branch {}` first.",
                        options.branch, options.branch
                    )
                })?;
            let exit_code = process::run_installed(&install, &options)?;
            launcher_state.last_selected_branch = Some(options.branch);
            state::save(&state_path, &launcher_state)?;
            if exit_code != 0 {
                return Err(format!("launched app exited with status code {exit_code}").into());
            }
        }
        LauncherCommand::ListInstalled => {
            if launcher_state.installs.is_empty() {
                println!("No installed builds yet.");
            } else {
                println!("Installed builds:");
                for install in launcher_state.installs {
                    println!(
                        "  {} @ {} ({})",
                        install.branch,
                        install.commit,
                        install.binary_path.display()
                    );
                }
            }
        }
    }

    Ok(())
}

fn resolve_repo_url(explicit: Option<String>, state: &models::LauncherState) -> String {
    explicit
        .or_else(detect_repo_url_from_cwd)
        .or_else(|| state.repo_url.clone())
        .unwrap_or_else(|| DEFAULT_REPO_URL.to_string())
}

fn detect_repo_url_from_cwd() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn upsert_install(installs: &mut Vec<InstalledBuild>, install: InstalledBuild) {
    if let Some(index) = installs
        .iter()
        .position(|candidate| candidate.branch == install.branch)
    {
        installs[index] = install;
    } else {
        installs.push(install);
        installs.sort_by(|a, b| a.branch.cmp(&b.branch));
    }
}

fn print_install_summary(install: &InstalledBuild) {
    println!("Installed branch: {}", install.branch);
    println!("Commit: {}", install.commit);
    println!("Source: {}", install.source_dir.display());
    println!("Binary: {}", install.binary_path.display());
    println!("Installed at: {}", install.installed_at_unix_secs);
}

pub fn build_run_options(
    branch: impl Into<String>,
    project: Option<PathBuf>,
    state_mode: Option<String>,
    window_mode: Option<String>,
    ui_scale: Option<f32>,
    extra_args: Vec<String>,
) -> RunLauncherOptions {
    RunLauncherOptions {
        branch: branch.into(),
        project,
        state_mode,
        window_mode,
        ui_scale,
        extra_args,
    }
}
