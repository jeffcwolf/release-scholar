use crate::config::Config;
use crate::report::Report;
use std::path::Path;

pub fn validate(project_dir: &Path, config: &Config, report: &mut Report) {
    for file in &config.required_files {
        let path = project_dir.join(file);
        if path.exists() {
            report.pass("Files", &format!("{} exists", file));
        } else {
            report.fail("Files", &format!("{} is missing", file));
        }
    }
}
