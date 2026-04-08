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
