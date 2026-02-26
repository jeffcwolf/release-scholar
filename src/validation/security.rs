use crate::report::Report;
use git2::Repository;
use regex::Regex;
use std::path::Path;

// severity: true = FAIL (high confidence), false = WARN (often false positive)
const SECRET_PATTERNS: &[(&str, &str, bool)] = &[
    (
        r"-----BEGIN\s+(RSA |DSA |EC |OPENSSH )?PRIVATE KEY-----",
        "Private key",
        true,
    ),
    (
        r#"(?i)(api[_-]?key|api[_-]?secret|access[_-]?token)\s*[:=]\s*['"]?\w{16,}"#,
        "API key/token",
        true,
    ),
    (
        r#"(?i)(password|passwd|pwd)\s*[:=]\s*['"]?.{8,}"#,
        "Password assignment",
        false,
    ),
    (r"AKIA[0-9A-Z]{16}", "AWS Access Key", true),
    (
        r"ghp_[A-Za-z0-9_]{36}",
        "GitHub Personal Access Token",
        true,
    ),
    (
        r"glpat-[A-Za-z0-9_\-]{20}",
        "GitLab Personal Access Token",
        true,
    ),
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

const RECOMMENDED_GITIGNORE_PATTERNS: &[&str] = &[".env", ".DS_Store", "*.pem", "*.key", "id_rsa"];

// Common build artifact patterns by ecosystem

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
    let patterns: Vec<(Regex, &str, bool)> = SECRET_PATTERNS
        .iter()
        .filter_map(|(pat, name, is_fail)| Regex::new(pat).ok().map(|r| (r, *name, *is_fail)))
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
            for (re, name, is_fail) in &patterns {
                if re.is_match(&content) {
                    if *is_fail {
                        report.fail(
                            "Security",
                            &format!("Possible {} found in tracked file: {}", name, path_str),
                        );
                    } else {
                        report.warn(
                            "Security",
                            &format!("Possible {} found in tracked file: {}", name, path_str),
                        );
                    }
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
                report.warn("Security", &format!("Sensitive file tracked: {}", path_str));
                found = true;
            }
        }
    }

    if !found {
        report.pass("Security", "No sensitive files tracked");
    }
}

fn scan_git_history(repo: &Repository, report: &mut Report) {
    // Only scan high-confidence patterns in git history
    let patterns: Vec<(Regex, &str)> = SECRET_PATTERNS
        .iter()
        .filter(|(_, _, is_fail)| *is_fail)
        .filter_map(|(pat, name, _)| Regex::new(pat).ok().map(|r| (r, *name)))
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
        report.warn(
            "Security",
            "Potential secrets found in git history (review recommended)",
        );
    } else {
        report.pass(
            "Security",
            &format!(
                "No secrets found in git history ({} commits scanned)",
                commits_checked
            ),
        );
    }
}

fn audit_gitignore(project_dir: &Path, report: &mut Report) {
    let gitignore_path = project_dir.join(".gitignore");
    if !gitignore_path.exists() {
        report.warn("Gitignore", ".gitignore not found");
        return;
    }

    let content = match std::fs::read_to_string(&gitignore_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Check security patterns
    let mut missing_security = Vec::new();
    for pattern in RECOMMENDED_GITIGNORE_PATTERNS {
        if !gitignore_contains(&content, pattern) {
            missing_security.push(*pattern);
        }
    }

    if missing_security.is_empty() {
        report.pass("Gitignore", "Covers common sensitive file patterns");
    } else {
        report.warn(
            "Gitignore",
            &format!("Missing security patterns: {}", missing_security.join(", ")),
        );
    }

    // Detect which ecosystems are present and check for relevant build artifact patterns
    let relevant = detect_relevant_artifacts(project_dir);
    let mut missing_artifacts: Vec<String> = Vec::new();
    for (pattern, description) in &relevant {
        if !gitignore_contains(&content, pattern) {
            missing_artifacts.push(format!("{} ({})", pattern, description));
        }
    }

    if missing_artifacts.is_empty() {
        report.pass(
            "Gitignore",
            "Covers build artifact patterns for detected languages",
        );
    } else {
        for missing in &missing_artifacts {
            report.warn(
                "Gitignore",
                &format!("Missing build artifact pattern: {}", missing),
            );
        }
    }
}

fn gitignore_contains(content: &str, pattern: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return false;
        }
        // Exact match or pattern is covered (e.g., "target/" covers "target/")
        trimmed == pattern || trimmed == pattern.trim_end_matches('/')
    })
}

/// Detect which ecosystems are present and return relevant artifact patterns
fn detect_relevant_artifacts(project_dir: &Path) -> Vec<(&'static str, &'static str)> {
    let mut relevant = Vec::new();

    // Always recommend release/
    relevant.push(("release/", "release-scholar build artifacts"));

    // Java/Maven
    if project_dir.join("pom.xml").exists() || project_dir.join("build.gradle").exists() {
        relevant.push(("target/", "Java/Maven build output"));
        relevant.push(("*.class", "Java compiled classes"));
    }

    // Python
    if project_dir.join("setup.py").exists()
        || project_dir.join("pyproject.toml").exists()
        || project_dir.join("requirements.txt").exists()
        || has_files_with_extension(project_dir, ".py")
    {
        relevant.push(("__pycache__/", "Python bytecode cache"));
        relevant.push(("*.pyc", "Python compiled files"));
        relevant.push(("*.egg-info", "Python package metadata"));
        relevant.push(("dist/", "Python distribution output"));
    }

    // Rust
    if project_dir.join("Cargo.toml").exists() {
        relevant.push(("target/", "Rust/Cargo build output"));
    }

    // Node.js
    if project_dir.join("package.json").exists() {
        relevant.push(("node_modules/", "Node.js dependencies"));
    }

    // General
    relevant.push((".DS_Store", "macOS metadata files"));

    // Deduplicate by pattern
    relevant.sort_by_key(|&(p, _)| p);
    relevant.dedup_by_key(|&mut (p, _)| p);

    relevant
}

fn has_files_with_extension(dir: &Path, ext: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(ext) {
                    return true;
                }
            }
        }
    }
    false
}
