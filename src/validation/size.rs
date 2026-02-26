use crate::report::Report;
use git2::Repository;
use std::path::Path;

const LARGE_FILE_THRESHOLD: u64 = 1_000_000; // 1 MB
const VERY_LARGE_FILE_THRESHOLD: u64 = 10_000_000; // 10 MB
const REPO_SIZE_WARN_THRESHOLD: u64 = 50_000_000; // 50 MB
const REPO_SIZE_FAIL_THRESHOLD: u64 = 200_000_000; // 200 MB

const BINARY_EXTENSIONS: &[&str] = &[
    ".zip", ".tar", ".gz", ".bz2", ".xz", ".7z", ".rar", ".jar", ".war", ".ear", ".exe", ".dll",
    ".so", ".dylib", ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".tiff", ".ico", ".svg", ".mp3",
    ".mp4", ".avi", ".mov", ".wav", ".flac", ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt",
    ".pptx", ".woff", ".woff2", ".ttf", ".eot", ".sqlite", ".db", ".min.js", ".min.css", ".map",
];

pub fn validate(project_dir: &Path, report: &mut Report) {
    let repo = match Repository::open(project_dir) {
        Ok(r) => r,
        Err(_) => return,
    };

    let index = match repo.index() {
        Ok(i) => i,
        Err(_) => return,
    };

    let mut total_size: u64 = 0;
    let mut large_files: Vec<(String, u64)> = Vec::new();
    let mut binary_files: Vec<(String, u64)> = Vec::new();
    let mut file_count: usize = 0;

    for entry in index.iter() {
        let path_str = String::from_utf8_lossy(&entry.path).to_string();
        let full_path = project_dir.join(&path_str);

        let size = match std::fs::metadata(&full_path) {
            Ok(m) => m.len(),
            Err(_) => continue,
        };

        total_size += size;
        file_count += 1;

        // Check for large files
        if size >= LARGE_FILE_THRESHOLD {
            large_files.push((path_str.clone(), size));
        }

        // Check for binary/vendor files that probably shouldn't be tracked
        let lower = path_str.to_lowercase();
        if BINARY_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) && size >= LARGE_FILE_THRESHOLD
        {
            binary_files.push((path_str, size));
        }
    }

    // Report total repo size
    let total_mb = total_size as f64 / 1_000_000.0;
    if total_size >= REPO_SIZE_FAIL_THRESHOLD {
        report.fail(
            "Size",
            &format!(
                "Tracked files total {:.1} MB — too large for a code repository",
                total_mb
            ),
        );
    } else if total_size >= REPO_SIZE_WARN_THRESHOLD {
        report.warn(
            "Size",
            &format!("Tracked files total {:.1} MB — consider reducing", total_mb),
        );
    } else {
        report.pass(
            "Size",
            &format!(
                "Tracked files total {:.1} MB ({} files)",
                total_mb, file_count
            ),
        );
    }

    // Report large files
    if large_files.is_empty() {
        report.pass("Size", "No large files detected (>1 MB)");
    } else {
        for (path, size) in &large_files {
            let size_mb = *size as f64 / 1_000_000.0;
            if *size >= VERY_LARGE_FILE_THRESHOLD {
                report.fail(
                    "Size",
                    &format!(
                        "{} is {:.1} MB — consider removing or using Git LFS",
                        path, size_mb
                    ),
                );
            } else {
                report.warn("Size", &format!("{} is {:.1} MB", path, size_mb));
            }
        }
    }

    // Report binary/vendor files
    if !binary_files.is_empty() {
        for (path, size) in &binary_files {
            let size_mb = *size as f64 / 1_000_000.0;
            report.warn(
                "Size",
                &format!(
                    "Binary/vendor file tracked: {} ({:.1} MB) — consider .gitignore or Git LFS",
                    path, size_mb
                ),
            );
        }
    }
}
