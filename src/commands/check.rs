use crate::config::Config;
use crate::report::Report;
use crate::validation;
use std::path::Path;

pub fn run(project_dir: &Path) -> Result<(), String> {
    let project_dir = std::fs::canonicalize(project_dir)
        .map_err(|e| format!("Invalid project directory: {}", e))?;
    let config = Config::load(&project_dir);
    let mut report = Report::new();

    // Git validation
    let git_info = validation::git::validate(&project_dir, &mut report);

    // File existence
    validation::files::validate(&project_dir, &config, &mut report);

    // Citation validation
    let version = git_info.as_ref().map(|g| g.version.as_str());
    validation::citation::validate(&project_dir, version, &mut report);

    // Security audit
    validation::security::validate(&project_dir, &mut report);

    // Size audit
    validation::size::validate(&project_dir, &mut report);

    report.print();

    if report.has_failures() {
        Err("Validation failed".to_string())
    } else {
        Ok(())
    }
}
