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
