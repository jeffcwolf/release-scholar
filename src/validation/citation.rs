use crate::report::Report;
use regex::Regex;
use std::path::Path;

pub fn validate(project_dir: &Path, expected_version: Option<&str>, report: &mut Report) {
    let cff_path = project_dir.join("CITATION.cff");
    if !cff_path.exists() {
        report.fail("Citation", "CITATION.cff not found");
        return;
    }

    let content = match std::fs::read_to_string(&cff_path) {
        Ok(c) => c,
        Err(e) => {
            report.fail("Citation", &format!("Cannot read CITATION.cff: {}", e));
            return;
        }
    };

    let doc: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(d) => d,
        Err(e) => {
            report.fail("Citation", &format!("Invalid YAML: {}", e));
            return;
        }
    };

    // cff-version
    if doc.get("cff-version").and_then(|v| v.as_str()).is_some() {
        report.pass("Citation", "cff-version present");
    } else {
        report.fail("Citation", "cff-version missing");
    }

    // title
    if doc.get("title").and_then(|v| v.as_str()).is_some() {
        report.pass("Citation", "title present");
    } else {
        report.fail("Citation", "title missing");
    }

    // authors
    let authors = doc.get("authors").and_then(|v| v.as_sequence());
    match authors {
        Some(list) if !list.is_empty() => {
            report.pass("Citation", &format!("{} author(s) found", list.len()));
            let orcid_re = Regex::new(r"^https://orcid\.org/\d{4}-\d{4}-\d{4}-\d{3}[\dX]$").unwrap();
            for (i, author) in list.iter().enumerate() {
                if author.get("family-names").and_then(|v| v.as_str()).is_none() {
                    report.fail("Citation", &format!("Author {} missing family-names", i + 1));
                }
                if let Some(orcid) = author.get("orcid").and_then(|v| v.as_str()) {
                    if orcid_re.is_match(orcid) {
                        report.pass("Citation", &format!("Author {} ORCID valid", i + 1));
                    } else {
                        report.fail("Citation", &format!("Author {} ORCID invalid: {}", i + 1, orcid));
                    }
                }
            }
        }
        _ => {
            report.fail("Citation", "No authors listed");
        }
    }

    // version matches git tag
    if let Some(expected) = expected_version {
        match doc.get("version").and_then(|v| v.as_str()) {
            Some(v) if v == expected => {
                report.pass("Citation", &format!("version matches git tag ({})", v));
            }
            Some(v) => {
                report.fail("Citation", &format!("version '{}' does not match git tag '{}'", v, expected));
            }
            None => {
                report.fail("Citation", "version missing");
            }
        }
    }

    // license
    if doc.get("license").and_then(|v| v.as_str()).is_some() {
        report.pass("Citation", "license present");
    } else {
        report.fail("Citation", "license missing");
    }

    // date-released
    if doc.get("date-released").is_some() {
        report.pass("Citation", "date-released present");
    } else {
        report.fail("Citation", "date-released missing");
    }
}
