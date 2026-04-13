use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

const CATALOG_RELEASE_TAG: &str = "launcher-catalog-latest";

#[derive(Debug, Clone)]
pub struct PublicCatalogSnapshot {
    pub branches: Vec<String>,
    pub branch_ahead_counts: HashMap<String, u64>,
    pub pr_titles: HashMap<String, String>,
    pub latest_release_tags: HashMap<String, String>,
    pub download_urls: HashMap<String, HashMap<String, String>>,
}

pub fn fetch_public_catalog_snapshot(
    repo_url: &str,
) -> Result<Option<PublicCatalogSnapshot>, Box<dyn std::error::Error>> {
    let Some((owner, repo)) = parse_github_repo(repo_url) else {
        return Ok(None);
    };
    let url = format!(
        "https://github.com/{owner}/{repo}/releases/download/{CATALOG_RELEASE_TAG}/launcher-catalog.json"
    );
    let client = Client::builder().build()?;
    let response = client
        .get(url)
        .header(USER_AGENT, "trekr-launcher")
        .send()?;
    if !response.status().is_success() {
        return Ok(None);
    }
    let payload = response.json::<LauncherCatalogPayload>()?;
    let mut branches = payload.branches;
    branches.sort();
    branches.dedup();
    Ok(Some(PublicCatalogSnapshot {
        branches,
        branch_ahead_counts: payload.ahead_by_main,
        pr_titles: payload.pr_titles,
        latest_release_tags: payload.latest_release_tags,
        download_urls: payload.download_urls,
    }))
}

pub fn list_remote_branches(repo_url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Some((owner, repo)) = parse_github_repo(repo_url) {
        let api_url = format!("https://api.github.com/repos/{owner}/{repo}/branches?per_page=100");
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
        if response.status().is_success() {
            let mut branches = response
                .json::<Vec<GithubBranch>>()?
                .into_iter()
                .map(|branch| branch.name)
                .collect::<Vec<_>>();
            branches.sort();
            branches.dedup();
            return Ok(branches);
        }
    }

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
    let Some((owner, repo)) = parse_github_repo(repo_url) else {
        return Ok(HashMap::new());
    };
    let client = Client::builder().build()?;

    let mut result = HashMap::new();
    for branch in branches {
        if branch.eq_ignore_ascii_case("main") {
            result.insert(branch.clone(), 1);
            continue;
        }
        let compare_url = format!(
            "https://api.github.com/repos/{owner}/{repo}/compare/main...{}",
            encode_compare_ref(branch)
        );
        let mut request = client
            .get(&compare_url)
            .header(USER_AGENT, "trekr-launcher")
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            if !token.trim().is_empty() {
                request = request.bearer_auth(token);
            }
        }
        let response = request.send()?;
        if response.status().is_success() {
            if let Ok(compare) = response.json::<GithubCompareResponse>() {
                result.insert(branch.clone(), compare.ahead_by);
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

fn encode_compare_ref(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('/', "%2F")
        .replace('#', "%23")
        .replace('?', "%3F")
        .replace(' ', "%20")
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

#[derive(Debug, Deserialize)]
struct GithubCompareResponse {
    ahead_by: u64,
}

#[derive(Debug, Deserialize)]
struct GithubBranch {
    name: String,
}

#[derive(Debug, Deserialize)]
struct LauncherCatalogPayload {
    #[serde(default)]
    branches: Vec<String>,
    #[serde(default)]
    ahead_by_main: HashMap<String, u64>,
    #[serde(default)]
    pr_titles: HashMap<String, String>,
    #[serde(default)]
    latest_release_tags: HashMap<String, String>,
    #[serde(default)]
    download_urls: HashMap<String, HashMap<String, String>>,
}
