use crate::launcher::models::InstalledBuild;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const APP_RELEASE_TAG_PREFIX: &str = "app-";

pub fn install_branch(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
    allow_source_build_fallback: bool,
    install_root: Option<&Path>,
) -> Result<InstalledBuild, Box<dyn std::error::Error>> {
    install_branch_with_progress(
        repo_url,
        branch,
        rebuild,
        allow_source_build_fallback,
        install_root,
        |_| {},
    )
}

pub fn latest_release_tag_for_branch(
    repo_url: &str,
    branch: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let releases = fetch_github_releases(repo_url)?;
    Ok(select_release_for_branch(&releases, branch).map(|release| release.tag_name.clone()))
}

pub fn install_branch_with_progress<F>(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
    allow_source_build_fallback: bool,
    install_root: Option<&Path>,
    mut progress: F,
) -> Result<InstalledBuild, Box<dyn std::error::Error>>
where
    F: FnMut(&str),
{
    let log_path = install_log_path(branch)?;
    write_log_header(&log_path, repo_url, branch)?;

    progress("Resolving GitHub release artifact");
    match install_from_release_artifact(repo_url, branch, install_root, &log_path, &mut progress) {
        Ok(install) => return Ok(install),
        Err(error) => {
            append_log(
                &log_path,
                &format!(
                    "\nartifact install failed: {error}\nallow_source_build_fallback={allow_source_build_fallback}\n"
                ),
            )?;
            if !allow_source_build_fallback {
                return Err(format!(
                    "artifact install failed: {error}\nsource-build fallback is disabled\nlog: {}",
                    log_path.display()
                )
                .into());
            }
        }
    }

    progress("Artifact install failed, falling back to source build");
    install_from_source(
        repo_url,
        branch,
        rebuild,
        install_root,
        &log_path,
        &mut progress,
    )
}

fn install_from_release_artifact<F>(
    repo_url: &str,
    branch: &str,
    install_root: Option<&Path>,
    log_path: &Path,
    progress: &mut F,
) -> Result<InstalledBuild, Box<dyn std::error::Error>>
where
    F: FnMut(&str),
{
    progress("Fetching release metadata");
    let releases = fetch_github_releases(repo_url)?;
    if releases.is_empty() {
        return Err("no GitHub releases found".into());
    }
    let (owner, repo) =
        parse_github_repo(repo_url).ok_or("repo url parsing failed while logging release api")?;
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=50");
    append_log(log_path, &format!("\nrelease api: {api_url}\n"))?;

    let client = github_client()?;

    let release = select_release_for_branch(&releases, branch)
        .ok_or("no matching GitHub release for branch")?;
    progress(&format!("Selected release {}", release.tag_name));

    let asset = select_asset_for_platform(&release.assets)
        .ok_or("no matching artifact asset for platform")?;
    append_log(
        log_path,
        &format!(
            "selected release: {} ({})\nselected asset: {} ({})\n",
            release.tag_name, release.name, asset.name, asset.browser_download_url
        ),
    )?;

    progress("Downloading artifact");
    let zip_path = download_asset(&client, asset, log_path)?;

    progress("Extracting artifact");
    let install_dir = artifact_install_dir(branch, &release.tag_name, install_root);
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;
    extract_archive(&zip_path, &install_dir)?;

    let binary_path = find_binary_recursive(&install_dir).ok_or_else(|| {
        format!(
            "artifact extracted but {} not found",
            expected_binary_name()
        )
    })?;

    let installed_at_unix_secs = unix_now();
    progress("Install completed from release artifact");
    Ok(InstalledBuild {
        branch: branch.to_string(),
        commit: release.tag_name.clone(),
        source_dir: install_dir,
        binary_path,
        installed_at_unix_secs,
    })
}

fn install_from_source<F>(
    repo_url: &str,
    branch: &str,
    rebuild: bool,
    install_root: Option<&Path>,
    log_path: &Path,
    progress: &mut F,
) -> Result<InstalledBuild, Box<dyn std::error::Error>>
where
    F: FnMut(&str),
{
    let source_root = source_checkout_root(install_root);
    fs::create_dir_all(&source_root)?;

    let source_dir = source_dir_for_branch(&source_root, branch);
    let target_dir = source_build_target_dir(branch, install_root);
    fs::create_dir_all(&target_dir)?;

    progress("Preparing source checkout");
    if source_dir.exists() {
        update_existing_checkout(&source_dir, branch, log_path)?;
    } else {
        clone_branch(repo_url, branch, &source_dir, log_path)?;
    }

    progress("Syncing git submodules");
    sync_submodules(&source_dir, log_path)?;

    let binary_path = source_release_binary_path(&target_dir);
    if rebuild || !binary_path.exists() {
        progress("Building source release binary");
        build_checkout(&source_dir, &target_dir, log_path)?;
    }
    if !binary_path.exists() {
        return Err(format!(
            "source build completed but binary was not found at {}\nlog: {}",
            binary_path.display(),
            log_path.display()
        )
        .into());
    }

    let commit = current_commit(&source_dir)?;
    let installed_at_unix_secs = unix_now();
    progress("Install completed from source build");
    Ok(InstalledBuild {
        branch: branch.to_string(),
        commit,
        source_dir,
        binary_path,
        installed_at_unix_secs,
    })
}

fn source_dir_for_branch(root: &Path, branch: &str) -> PathBuf {
    root.join(sanitize_segment(branch))
}

fn source_checkout_root(install_root: Option<&Path>) -> PathBuf {
    if let Some(root) = install_root {
        return root.join("sources");
    }
    PathBuf::from("artifacts/launcher/sources")
}

fn source_build_target_dir(branch: &str, install_root: Option<&Path>) -> PathBuf {
    let safe = sanitize_segment(branch);
    if let Some(root) = install_root {
        return root.join("source-target").join(safe);
    }
    if cfg!(windows) {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join("trekr-launcher-target")
                .join(safe)
                .join("source");
        }
    }
    PathBuf::from("artifacts/launcher/target").join(safe)
}

fn artifact_install_dir(branch: &str, release_tag: &str, install_root: Option<&Path>) -> PathBuf {
    let branch_safe = sanitize_segment(branch);
    let tag_safe = sanitize_segment(release_tag);
    if let Some(root) = install_root {
        return root.join("builds").join(branch_safe).join(tag_safe);
    }
    if cfg!(windows) {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local_app_data)
                .join("trekr-launcher-builds")
                .join(branch_safe)
                .join(tag_safe);
        }
    }
    PathBuf::from("artifacts/launcher/builds")
        .join(branch_safe)
        .join(tag_safe)
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

fn source_release_binary_path(target_dir: &Path) -> PathBuf {
    target_dir.join("release").join(expected_binary_name())
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

fn github_client() -> Result<Client, Box<dyn std::error::Error>> {
    Ok(Client::builder().build()?)
}

fn fetch_github_releases(repo_url: &str) -> Result<Vec<GithubRelease>, Box<dyn std::error::Error>> {
    let (owner, repo) = parse_github_repo(repo_url).ok_or_else(|| {
        format!(
            "unsupported repo url for release artifact install: {repo_url} (expected github url)"
        )
    })?;
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=50");
    let client = github_client()?;
    let mut request = client
        .get(&api_url)
        .header(USER_AGENT, "trekr-launcher")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            request = request.bearer_auth(token);
        }
    }
    let response = request.send()?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        return Err(format!("release metadata request failed: {status} {}", body.trim()).into());
    }
    Ok(response.json::<Vec<GithubRelease>>()?)
}

fn download_asset(
    client: &Client,
    asset: &GithubAsset,
    log_path: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cache_dir = artifact_cache_dir()?;
    let file_path = cache_dir.join(format!("{}-{}", unix_now(), sanitize_segment(&asset.name)));
    let mut response = client
        .get(&asset.browser_download_url)
        .header(USER_AGENT, "trekr-launcher")
        .header(ACCEPT, "application/octet-stream")
        .send()?
        .error_for_status()?;
    let mut file = fs::File::create(&file_path)?;
    let mut bytes = Vec::new();
    response.read_to_end(&mut bytes)?;
    file.write_all(&bytes)?;
    append_log(
        log_path,
        &format!("downloaded asset to {}\n", file_path.display()),
    )?;
    Ok(file_path)
}

fn artifact_cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = if cfg!(windows) {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            PathBuf::from(local_app_data).join("trekr-launcher-cache")
        } else {
            PathBuf::from("artifacts/launcher/cache")
        }
    } else {
        PathBuf::from("artifacts/launcher/cache")
    };
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn extract_zip_archive(
    zip_path: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let Some(rel_path) = entry.enclosed_name().map(|path| path.to_owned()) else {
            continue;
        };
        let out_path = destination.join(rel_path);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out_file = fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }
    Ok(())
}

fn extract_archive(
    archive_path: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let lowercase = archive_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_lowercase())
        .unwrap_or_default();
    if lowercase.ends_with(".zip") {
        return extract_zip_archive(archive_path, destination);
    }
    if lowercase.ends_with(".tar.gz") || lowercase.ends_with(".tgz") {
        return extract_tar_gz_archive(archive_path, destination);
    }
    if lowercase.ends_with(".tar") {
        let file = fs::File::open(archive_path)?;
        let mut archive = tar::Archive::new(file);
        archive.unpack(destination)?;
        return Ok(());
    }
    Err(format!("unsupported archive format: {}", archive_path.display()).into())
}

fn extract_tar_gz_archive(
    archive_path: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::open(archive_path)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(destination)?;
    Ok(())
}

fn find_binary_recursive(root: &Path) -> Option<PathBuf> {
    let target = expected_binary_name();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path).ok()?;
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case(target))
            {
                return Some(entry_path);
            }
        }
    }
    None
}

fn expected_binary_name() -> &'static str {
    if cfg!(windows) { "trekr.exe" } else { "trekr" }
}

fn parse_github_repo(repo_url: &str) -> Option<(String, String)> {
    if let Some(trimmed) = repo_url.strip_prefix("https://github.com/") {
        return parse_owner_repo(trimmed);
    }
    if let Some(trimmed) = repo_url.strip_prefix("http://github.com/") {
        return parse_owner_repo(trimmed);
    }
    if let Some(trimmed) = repo_url.strip_prefix("git@github.com:") {
        return parse_owner_repo(trimmed);
    }
    None
}

fn parse_owner_repo(value: &str) -> Option<(String, String)> {
    let trimmed = value.trim_end_matches(".git").trim_matches('/');
    let mut parts = trimmed.split('/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

fn select_release_for_branch<'a>(
    releases: &'a [GithubRelease],
    branch: &str,
) -> Option<&'a GithubRelease> {
    let branch_token = normalize_branch_token(branch);
    let tagged_releases: Vec<&GithubRelease> = releases
        .iter()
        .filter(|release| {
            !release.draft
                && release
                    .tag_name
                    .to_lowercase()
                    .starts_with(APP_RELEASE_TAG_PREFIX)
        })
        .collect();
    let candidates = if tagged_releases.is_empty() {
        releases.iter().filter(|release| !release.draft).collect()
    } else {
        tagged_releases
    };

    let mut best: Option<(&GithubRelease, i32)> = None;
    for release in candidates.iter().copied() {
        let normalized_tag = normalize_branch_token(&release.tag_name);
        let normalized_name = normalize_branch_token(&release.name);
        let haystack = format!("{} {}", normalized_tag, normalized_name);
        let mut score = 0;
        if haystack.contains(&branch_token) {
            score += 3;
        }
        if branch.eq_ignore_ascii_case("main")
            && (haystack.contains("main") || haystack.contains("stable"))
        {
            score += 2;
        }
        if score == 0 && branch.eq_ignore_ascii_case("main") {
            score = 1;
        }
        match best {
            Some((_, best_score)) if score <= best_score => {}
            _ => best = Some((release, score)),
        }
    }
    best.map(|(release, _)| release)
        .or_else(|| candidates.first().copied())
}

fn select_asset_for_platform<'a>(assets: &'a [GithubAsset]) -> Option<&'a GithubAsset> {
    let platform_tokens: &[&str] = if cfg!(windows) {
        &["windows", "win64", "x86_64-pc-windows-msvc"]
    } else if cfg!(target_os = "macos") {
        &["macos", "darwin", "apple"]
    } else {
        &["linux", "x86_64-unknown-linux-gnu"]
    };

    let is_supported_archive = |name: &str| {
        name.ends_with(".zip")
            || name.ends_with(".tar.gz")
            || name.ends_with(".tgz")
            || name.ends_with(".tar")
    };
    assets
        .iter()
        .find(|asset| {
            let name = asset.name.to_lowercase();
            is_supported_archive(&name) && platform_tokens.iter().any(|token| name.contains(token))
        })
        .or_else(|| {
            assets.iter().find(|asset| {
                let name = asset.name.to_lowercase();
                is_supported_archive(&name)
            })
        })
}

fn normalize_branch_token(branch: &str) -> String {
    branch
        .to_lowercase()
        .replace('/', "-")
        .replace('_', "-")
        .replace(' ', "-")
}

fn install_log_path(branch: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let log_dir = PathBuf::from("artifacts/launcher/logs");
    fs::create_dir_all(&log_dir)?;
    Ok(log_dir.join(format!(
        "install-{}-{}.log",
        sanitize_segment(branch),
        unix_now()
    )))
}

fn write_log_header(
    log_path: &Path,
    repo_url: &str,
    branch: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    append_log(
        log_path,
        &format!(
            "# trekr launcher install log\nrepo: {repo_url}\nbranch: {branch}\nstarted_at: {}\n",
            unix_now()
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

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}
