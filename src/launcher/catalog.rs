use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

pub fn list_remote_branches(repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(["ls-remote", "--heads", repo_url])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("failed to list branches: {}", stderr.trim()).into());
    }

    let mut branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (_, reference) = line.split_once('\t')?;
            reference
                .strip_prefix("refs/heads/")
                .map(|name| name.to_string())
        })
        .collect::<Vec<_>>();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

pub fn fetch_branch_ahead_counts_vs_main(
    repo_url: &str,
    branches: &[String],
) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let metadata_repo = prepare_launcher_git_metadata_repo(repo_url)?;
    let _ = Command::new("git")
        .arg("-C")
        .arg(&metadata_repo)
        .args([
            "fetch",
            "--prune",
            "--quiet",
            "origin",
            "+refs/heads/*:refs/remotes/origin/*",
        ])
        .output();

    let mut result = HashMap::new();
    for branch in branches {
        if branch.eq_ignore_ascii_case("main") {
            result.insert(branch.clone(), 1);
            continue;
        }
        let branch_ref = format!("origin/{branch}");
        let rev_range = format!("origin/main...{branch_ref}");
        let output = Command::new("git")
            .arg("-C")
            .arg(&metadata_repo)
            .args(["rev-list", "--left-right", "--count"])
            .arg(&rev_range)
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let counts = String::from_utf8_lossy(&output.stdout);
                let mut parts = counts.split_whitespace();
                let _behind = parts.next();
                if let Some(ahead) = parts.next().and_then(|value| value.parse::<u64>().ok()) {
                    result.insert(branch.clone(), ahead);
                }
            }
        }
    }
    Ok(result)
}

pub fn list_open_pr_titles(
    repo_url: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let Some((owner, repo)) = parse_github_repo(repo_url) else {
        return Ok(HashMap::new());
    };
    let api_url =
        format!("https://api.github.com/repos/{owner}/{repo}/pulls?state=open&per_page=100");
    let client = Client::builder().build()?;
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
        return Ok(HashMap::new());
    }
    let pulls = response.json::<Vec<GithubPullRequest>>()?;
    Ok(pulls
        .into_iter()
        .map(|pull| (pull.head.reference, pull.title))
        .collect())
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

fn prepare_launcher_git_metadata_repo(
    repo_url: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = launcher_metadata_repo_dir();
    std::fs::create_dir_all(&repo_dir)?;
    let git_dir = repo_dir.join(".git");
    if !git_dir.exists() {
        let status = Command::new("git").arg("init").arg(&repo_dir).status()?;
        if !status.success() {
            return Err("failed to initialize launcher metadata git repo".into());
        }
    }

    let remote_url = Command::new("git")
        .arg("-C")
        .arg(&repo_dir)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string());

    match remote_url {
        Some(existing) if existing == repo_url => {}
        Some(_) => {
            let _ = Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .args(["remote", "set-url", "origin", repo_url])
                .status();
        }
        None => {
            let _ = Command::new("git")
                .arg("-C")
                .arg(&repo_dir)
                .args(["remote", "add", "origin", repo_url])
                .status();
        }
    }

    Ok(repo_dir)
}

fn launcher_metadata_repo_dir() -> std::path::PathBuf {
    if cfg!(windows) {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            return std::path::PathBuf::from(local_app_data)
                .join("trekr-launcher-cache")
                .join("branch-metadata");
        }
    }
    std::path::PathBuf::from("artifacts/launcher/cache/branch-metadata")
}

#[derive(Debug, Deserialize)]
struct GithubPullRequest {
    title: String,
    head: GithubPullRequestHead,
}

#[derive(Debug, Deserialize)]
struct GithubPullRequestHead {
    #[serde(rename = "ref")]
    reference: String,
}
