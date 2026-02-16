use crate::config::Config;
use colored::Colorize;
use reqwest::blocking::Client;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
struct PushMirrorRequest {
    remote_address: String,
    remote_username: String,
    remote_password: String,
    interval: String,
    sync_on_commit: bool,
}

pub fn run(project_dir: &Path) -> Result<(), String> {
    let project_dir = std::fs::canonicalize(project_dir)
        .map_err(|e| format!("Invalid project directory: {}", e))?;
    let config = Config::load(&project_dir);

    let mirrors = config.mirrors.as_ref().ok_or(
        "No [mirrors] section in config. Add it to your global config at:\n  \
         ~/Library/Application Support/release-scholar/config.toml (macOS)\n  \
         ~/.config/release-scholar/config.toml (Linux)",
    )?;

    let codeberg_token = mirrors
        .codeberg_token
        .as_deref()
        .ok_or("codeberg_token not set in [mirrors] config")?;
    let codeberg_user = mirrors
        .codeberg_user
        .as_deref()
        .ok_or("codeberg_user not set in [mirrors] config")?;

    // Determine repo name from directory
    let repo_name = project_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    println!(
        "\n{} Setting up push mirrors for {}/{}...\n",
        ">>>".bold(),
        codeberg_user,
        repo_name.bold()
    );

    let client = Client::builder()
        .user_agent(format!("release-scholar/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| format!("Cannot create HTTP client: {}", e))?;

    // Check existing mirrors first
    let existing = get_existing_mirrors(&client, codeberg_user, &repo_name, codeberg_token)?;

    // GitHub mirror
    if let (Some(gh_user), Some(gh_token)) = (&mirrors.github_user, &mirrors.github_token) {
        let gh_url = format!("https://github.com/{}/{}.git", gh_user, repo_name);
        if existing.iter().any(|url| url.contains("github.com")) {
            println!(
                "  {} GitHub mirror already exists — skipping",
                "OK".green()
            );
        } else {
            print!("  Adding GitHub mirror... ");
            add_push_mirror(
                &client,
                codeberg_user,
                &repo_name,
                codeberg_token,
                &gh_url,
                gh_user,
                gh_token,
            )?;
            println!("{}", "done".green());
            println!("    → {}", gh_url);
        }
    } else {
        println!(
            "  {} GitHub: skipped (github_user/github_token not configured)",
            "—".dimmed()
        );
    }

    // GitLab mirror
    if let (Some(gl_user), Some(gl_token)) = (&mirrors.gitlab_user, &mirrors.gitlab_token) {
        let gl_url = format!("https://gitlab.com/{}/{}.git", gl_user, repo_name);
        if existing.iter().any(|url| url.contains("gitlab.com")) {
            println!(
                "  {} GitLab mirror already exists — skipping",
                "OK".green()
            );
        } else {
            print!("  Adding GitLab mirror... ");
            add_push_mirror(
                &client,
                codeberg_user,
                &repo_name,
                codeberg_token,
                &gl_url,
                gl_user,
                gl_token,
            )?;
            println!("{}", "done".green());
            println!("    → {}", gl_url);
        }
    } else {
        println!(
            "  {} GitLab: skipped (gitlab_user/gitlab_token not configured)",
            "—".dimmed()
        );
    }

    println!(
        "\n  {} Mirrors will sync every 8 hours and on push.\n",
        "OK".green().bold()
    );

    Ok(())
}

fn get_existing_mirrors(
    client: &Client,
    owner: &str,
    repo: &str,
    token: &str,
) -> Result<Vec<String>, String> {
    let url = format!(
        "https://codeberg.org/api/v1/repos/{}/{}/push_mirrors",
        owner, repo
    );
    let resp = client
        .get(&url)
        .header("Authorization", format!("token {}", token))
        .send()
        .map_err(|e| format!("HTTP error listing mirrors: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("Codeberg API error {} listing mirrors: {}", status, body));
    }

    let mirrors: Vec<serde_json::Value> = resp.json().unwrap_or_default();
    Ok(mirrors
        .iter()
        .filter_map(|m| m.get("remote_address").and_then(|v| v.as_str()).map(String::from))
        .collect())
}

fn add_push_mirror(
    client: &Client,
    owner: &str,
    repo: &str,
    codeberg_token: &str,
    remote_url: &str,
    remote_user: &str,
    remote_token: &str,
) -> Result<(), String> {
    let url = format!(
        "https://codeberg.org/api/v1/repos/{}/{}/push_mirrors",
        owner, repo
    );

    let body = PushMirrorRequest {
        remote_address: remote_url.to_string(),
        remote_username: remote_user.to_string(),
        remote_password: remote_token.to_string(),
        interval: "8h0m0s".to_string(),
        sync_on_commit: true,
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("token {}", codeberg_token))
        .json(&body)
        .send()
        .map_err(|e| format!("HTTP error adding mirror: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().unwrap_or_default();
        return Err(format!("Codeberg API error {} adding mirror: {}", status, body));
    }

    Ok(())
}
