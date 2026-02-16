use crate::report::Report;
use git2::Repository;
use regex::Regex;
use std::path::Path;

pub struct GitInfo {
    pub version: String,
    pub tag: String,
}

pub fn validate(project_dir: &Path, report: &mut Report) -> Option<GitInfo> {
    let repo = match Repository::open(project_dir) {
        Ok(r) => r,
        Err(e) => {
            report.fail("Git", &format!("Cannot open repository: {}", e));
            return None;
        }
    };

    // Check working directory is clean
    let statuses = repo.statuses(None);
    match statuses {
        Ok(s) => {
            let dirty: Vec<String> = s
                .iter()
                .filter(|e| {
                    e.status() != git2::Status::IGNORED
                })
                .map(|e| e.path().unwrap_or("?").to_string())
                .collect();
            if dirty.is_empty() {
                report.pass("Git", "Working directory is clean");
            } else {
                report.warn(
                    "Git",
                    &format!(
                        "Working directory has {} uncommitted change(s): {}",
                        dirty.len(),
                        dirty.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
                    ),
                );
            }
        }
        Err(e) => {
            report.fail("Git", &format!("Cannot check status: {}", e));
        }
    }

    // Find semver tag on HEAD
    let head = match repo.head() {
        Ok(h) => h,
        Err(e) => {
            report.fail("Git", &format!("Cannot read HEAD: {}", e));
            return None;
        }
    };
    let head_oid = head.target().unwrap();

    let semver_re = Regex::new(r"^v(\d+\.\d+\.\d+)$").unwrap();
    let tag_names = match repo.tag_names(None) {
        Ok(t) => t,
        Err(e) => {
            report.fail("Git", &format!("Cannot list tags: {}", e));
            return None;
        }
    };

    let mut found_tag: Option<(String, String)> = None;
    for i in 0..tag_names.len() {
        let name = match tag_names.get(i) {
            Some(n) => n,
            None => continue,
        };
        if let Some(caps) = semver_re.captures(name) {
            // Resolve tag to commit
            let tag_oid = match repo.revparse_single(&format!("refs/tags/{}", name)) {
                Ok(obj) => obj.peel_to_commit().map(|c| c.id()).unwrap_or(obj.id()),
                Err(_) => continue,
            };
            if tag_oid == head_oid {
                found_tag = Some((name.to_string(), caps[1].to_string()));
                break;
            }
        }
    }

    match found_tag {
        Some((tag, version)) => {
            report.pass("Git", &format!("HEAD is tagged: {} (version {})", tag, version));
            Some(GitInfo { version, tag })
        }
        None => {
            report.fail("Git", "HEAD has no semver tag (expected vX.Y.Z)");
            None
        }
    }
}
