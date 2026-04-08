use crate::launcher::models::InstalledBuild;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn install_branch(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
) -> Result<InstalledBuild, Box<dyn std::error::Error>> {
    let root_dir = PathBuf::from("artifacts/launcher/sources");
    fs::create_dir_all(&root_dir)?;

    let source_dir = source_dir_for_branch(&root_dir, branch);
    if source_dir.exists() {
        update_existing_checkout(&source_dir, branch)?;
    } else {
        clone_branch(repo_url, branch, &source_dir)?;
    }

    let binary_path = release_binary_path(&source_dir);
    if rebuild || !binary_path.exists() {
        build_checkout(&source_dir)?;
    }
    if !binary_path.exists() {
        return Err(format!(
            "build completed but binary was not found at {}",
            binary_path.display()
        )
        .into());
    }

    let commit = current_commit(&source_dir)?;
    let installed_at_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    Ok(InstalledBuild {
        branch: branch.to_string(),
        commit,
        source_dir,
        binary_path,
        installed_at_unix_secs,
    })
}

fn source_dir_for_branch(root: &Path, branch: &str) -> PathBuf {
    root.join(branch.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"))
}

fn clone_branch(
    repo_url: &str,
    branch: &str,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("git")
            .args(["clone", "--depth", "1", "--branch", branch, repo_url])
            .arg(destination),
        "git clone",
    )
}

fn update_existing_checkout(
    source_dir: &Path,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["fetch", "--depth", "1", "origin", branch]),
        "git fetch",
    )?;
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["checkout", branch]),
        "git checkout",
    )?;
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["reset", "--hard", "FETCH_HEAD"]),
        "git reset",
    )
}

fn build_checkout(source_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--bin")
            .arg("trekr")
            .current_dir(source_dir),
        "cargo build",
    )
}

fn release_binary_path(source_dir: &Path) -> PathBuf {
    let binary_name = if cfg!(windows) { "trekr.exe" } else { "trekr" };
    source_dir.join("target").join("release").join(binary_name)
}

fn current_commit(source_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(source_dir)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("failed to read commit: {}", stderr.trim()).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_checked(command: &mut Command, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "{label} failed.\nstdout:\n{}\nstderr:\n{}",
        stdout.trim(),
        stderr.trim()
    )
    .into())
}
