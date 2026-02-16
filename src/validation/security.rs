use crate::report::Report;
use git2::Repository;
use regex::Regex;
use std::path::Path;

const SECRET_PATTERNS: &[(&str, &str)] = &[
    (r"-----BEGIN\s+(RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----", "Private key"),
    (r#"(?i)(api[_-]?key|api[_-]?secret|access[_-]?token)\s*[:=]\s*['"]?\w{16,}"#, "API key/token"),
    (r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]?.{8,}"#, "Password assignment"),
    (r"AKIA[0-9A-Z]{16}", "AWS Access Key"),
    (r"ghp_[A-Za-z0-9_]{36}", "GitHub Personal Access Token"),
    (r"glpat-[A-Za-z0-9_\-]{20}", "GitLab Personal Access Token"),
];

const SENSITIVE_FILE_PATTERNS: &[&str] = &[
    ".env",
    ".pem",
    ".key",
    "id_rsa",
    "id_dsa",
    "id_ed25519",
    "credentials.json",
    ".sqlite",
    ".DS_Store",
    ".p12",
    ".pfx",
];

const RECOMMENDED_GITIGNORE_PATTERNS: &[&str] = &[
    ".env",
    ".DS_Store",
    "*.pem",
    "*.key",
    "id_rsa",
];

pub fn validate(project_dir: &Path, report: &mut Report) {
    let repo = match Repository::open(project_dir) {
        Ok(r) => r,
        Err(_) => {
            report.fail("Security", "Cannot open repository for security scan");
            return;
        }
    };

    scan_tracked_files_for_secrets(&repo, project_dir, report);
    scan_sensitive_files(&repo, report);
    scan_git_history(&repo, report);
    audit_gitignore(project_dir, report);
}

fn scan_tracked_files_for_secrets(repo: &Repository, project_dir: &Path, report: &mut Report) {
    let patterns: Vec<(Regex, &str)> = SECRET_PATTERNS
        .iter()
        .filter_map(|(pat, name)| Regex::new(pat).ok().map(|r| (r, *name)))
        .collect();

    let index = match repo.index() {
        Ok(i) => i,
        Err(_) => return,
    };

    let mut found_secrets = false;
    for entry in index.iter() {
        let path_str = String::from_utf8_lossy(&entry.path);
        let full_path = project_dir.join(&*path_str);

        // Only scan text-like files
        if let Ok(content) = std::fs::read_to_string(&full_path) {
            for (re, name) in &patterns {
                if re.is_match(&content) {
                    report.fail(
                        "Security",
                        &format!("Possible {} found in tracked file: {}", name, path_str),
                    );
                    found_secrets = true;
                }
            }
        }
    }

    if !found_secrets {
        report.pass("Security", "No secrets detected in tracked files");
    }
}

fn scan_sensitive_files(repo: &Repository, report: &mut Report) {
    let index = match repo.index() {
        Ok(i) => i,
        Err(_) => return,
    };

    let mut found = false;
    for entry in index.iter() {
        let path_str = String::from_utf8_lossy(&entry.path).to_string();
        let filename = Path::new(&path_str)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for pattern in SENSITIVE_FILE_PATTERNS {
            if filename == *pattern || filename.ends_with(pattern) {
                report.warn(
                    "Security",
                    &format!("Sensitive file tracked: {}", path_str),
                );
                found = true;
            }
        }
    }

    if !found {
        report.pass("Security", "No sensitive files tracked");
    }
}

fn scan_git_history(repo: &Repository, report: &mut Report) {
    let patterns: Vec<(Regex, &str)> = SECRET_PATTERNS
        .iter()
        .filter_map(|(pat, name)| Regex::new(pat).ok().map(|r| (r, *name)))
        .collect();

    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return,
    };
    revwalk.push_head().ok();

    let mut found_in_history = false;
    let mut commits_checked = 0;
    let max_commits = 100;

    for oid in revwalk {
        let oid = match oid {
            Ok(o) => o,
            Err(_) => continue,
        };
        if commits_checked >= max_commits {
            break;
        }
        commits_checked += 1;

        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let tree = match commit.tree() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // Get parent tree for diff
        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

        let diff = match repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) {
            Ok(d) => d,
            Err(_) => continue,
        };

        diff.foreach(
            &mut |_, _| true,
            None,
            None,
            Some(&mut |_delta, _hunk, line| {
                if line.origin() == '+' || line.origin() == ' ' {
                    let content = String::from_utf8_lossy(line.content());
                    for (re, name) in &patterns {
                        if re.is_match(&content) {
                            if !found_in_history {
                                found_in_history = true;
                            }
                            let _ = name; // just flag once
                        }
                    }
                }
                true
            }),
        )
        .ok();
    }

    if found_in_history {
        report.warn("Security", "Potential secrets found in git history (review recommended)");
    } else {
        report.pass("Security", &format!("No secrets found in git history ({} commits scanned)", commits_checked));
    }
}

fn audit_gitignore(project_dir: &Path, report: &mut Report) {
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        report.warn("Security", ".gitignore not found");
        return;
    }

    let content = match std::fs::read_to_string(&gitignore_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut missing = Vec::new();
    for pattern in RECOMMENDED_GITIGNORE_PATTERNS {
        if !content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == *pattern || trimmed.starts_with(pattern)
        }) {
            missing.push(*pattern);
        }
    }

    if missing.is_empty() {
        report.pass("Security", ".gitignore covers common sensitive patterns");
    } else {
        report.warn(
            "Security",
            &format!(".gitignore missing recommended patterns: {}", missing.join(", ")),
        );
    }
}
