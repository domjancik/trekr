use crate::launcher::models::InstalledBuild;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn install_branch(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
) -> Result<InstalledBuild, Box<dyn std::error::Error>> {
    install_branch_with_progress(repo_url, branch, rebuild, |_| {})
}

pub fn install_branch_with_progress<F>(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
    mut progress: F,
) -> Result<InstalledBuild, Box<dyn std::error::Error>>
where
    F: FnMut(&str),
{
    let source_root = PathBuf::from("artifacts/launcher/sources");
    fs::create_dir_all(&source_root)?;

    let source_dir = source_dir_for_branch(&source_root, branch);
    let target_dir = launcher_target_dir(branch);
    fs::create_dir_all(&target_dir)?;

    let log_path = install_log_path(branch)?;
    write_log_header(&log_path, repo_url, branch, &source_dir, &target_dir)?;

    progress("Preparing branch checkout");
    if source_dir.exists() {
        update_existing_checkout(&source_dir, branch, &log_path)?;
    } else {
        clone_branch(repo_url, branch, &source_dir, &log_path)?;
    }
    progress("Syncing git submodules");
    sync_submodules(&source_dir, &log_path)?;

    let binary_path = release_binary_path(&target_dir);
    if rebuild || !binary_path.exists() {
        progress("Building release binary");
        build_checkout(&source_dir, &target_dir, &log_path)?;
    }
    if !binary_path.exists() {
        return Err(format!(
            "build completed but binary was not found at {}\nlog: {}",
            binary_path.display(),
            log_path.display()
        )
        .into());
    }

    let commit = current_commit(&source_dir)?;
    let installed_at_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    progress("Install completed");

    Ok(InstalledBuild {
        branch: branch.to_string(),
        commit,
        source_dir,
        binary_path,
        installed_at_unix_secs,
    })
}

fn sync_submodules(source_dir: &Path, log_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("git").arg("-C").arg(source_dir).args([
            "submodule",
            "update",
            "--init",
            "--recursive",
            "vendor/ableton-link",
        ]),
        "git submodule update",
        log_path,
    )
}

fn source_dir_for_branch(root: &Path, branch: &str) -> PathBuf {
    root.join(sanitize_segment(branch))
}

fn clone_branch(
    repo_url: &str,
    branch: &str,
    destination: &Path,
    log_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("git")
            .args(["clone", "--depth", "1", "--branch", branch, repo_url])
            .arg(destination),
        "git clone",
        log_path,
    )
}

fn update_existing_checkout(
    source_dir: &Path,
    branch: &str,
    log_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["fetch", "--depth", "1", "origin", branch]),
        "git fetch",
        log_path,
    )?;
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["checkout", branch]),
        "git checkout",
        log_path,
    )?;
    run_checked(
        Command::new("git")
            .arg("-C")
            .arg(source_dir)
            .args(["reset", "--hard", "FETCH_HEAD"]),
        "git reset",
        log_path,
    )
}

fn build_checkout(
    source_dir: &Path,
    target_dir: &Path,
    log_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    run_checked(
        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg("--bin")
            .arg("trekr")
            .arg("--target-dir")
            .arg(target_dir)
            .current_dir(source_dir),
        "cargo build",
        log_path,
    )
}

fn release_binary_path(target_dir: &Path) -> PathBuf {
    let binary_name = if cfg!(windows) { "trekr.exe" } else { "trekr" };
    target_dir.join("release").join(binary_name)
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

fn run_checked(
    command: &mut Command,
    label: &str,
    log_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    append_log(log_path, &format!("\n## {label}\n$ {command:?}\n"))?;
    let output = command.output()?;
    append_log(
        log_path,
        &format!(
            "status: {}\nstdout:\n{}\nstderr:\n{}\n",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "{label} failed.\nstdout:\n{}\nstderr:\n{}\nlog: {}",
        stdout.trim(),
        stderr.trim(),
        log_path.display()
    )
    .into())
}

fn launcher_target_dir(branch: &str) -> PathBuf {
    let safe = sanitize_segment(branch);
    if cfg!(windows) {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join("trekr-launcher-target")
                .join(safe);
        }
    }
    PathBuf::from("artifacts/launcher/target").join(safe)
}

fn install_log_path(branch: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let log_dir = PathBuf::from("artifacts/launcher/logs");
    fs::create_dir_all(&log_dir)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    Ok(log_dir.join(format!("install-{}-{ts}.log", sanitize_segment(branch))))
}

fn write_log_header(
    log_path: &Path,
    repo_url: &str,
    branch: &str,
    source_dir: &Path,
    target_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    append_log(
        log_path,
        &format!(
            "# trekr launcher install log\nrepo: {repo_url}\nbranch: {branch}\nsource: {}\ntarget: {}\n",
            source_dir.display(),
            target_dir.display()
        ),
    )
}

fn append_log(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn sanitize_segment(value: &str) -> String {
    value.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}
