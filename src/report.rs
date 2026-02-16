use colored::Colorize;

#[derive(Debug, Clone)]
pub enum Status {
    Pass,
    Fail,
    Warn,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub category: String,
    pub message: String,
    pub status: Status,
}

pub struct Report {
    pub results: Vec<CheckResult>,
}

impl Report {
    pub fn new() -> Self {
        Report { results: Vec::new() }
    }

    pub fn add(&mut self, category: &str, message: &str, status: Status) {
        self.results.push(CheckResult {
            category: category.to_string(),
            message: message.to_string(),
            status,
        });
    }

    pub fn pass(&mut self, category: &str, message: &str) {
        self.add(category, message, Status::Pass);
    }

    pub fn fail(&mut self, category: &str, message: &str) {
        self.add(category, message, Status::Fail);
    }

    pub fn warn(&mut self, category: &str, message: &str) {
        self.add(category, message, Status::Warn);
    }

    pub fn has_failures(&self) -> bool {
        self.results.iter().any(|r| matches!(r.status, Status::Fail))
    }

    pub fn print(&self) {
        println!("\n{}", "═══ Release Scholar Report ═══".bold());
        println!();

        for result in &self.results {
            let icon = match result.status {
                Status::Pass => "[PASS]".green().bold(),
                Status::Fail => "[FAIL]".red().bold(),
                Status::Warn => "[WARN]".yellow().bold(),
            };
            println!("  {} {}: {}", icon, result.category.bold(), result.message);
        }

        let passes = self.results.iter().filter(|r| matches!(r.status, Status::Pass)).count();
        let fails = self.results.iter().filter(|r| matches!(r.status, Status::Fail)).count();
        let warns = self.results.iter().filter(|r| matches!(r.status, Status::Warn)).count();

        println!();
        println!(
            "  {} passed, {} failed, {} warnings",
            passes.to_string().green(),
            if fails > 0 { fails.to_string().red() } else { fails.to_string().normal() },
            warns.to_string().yellow()
        );

        if fails > 0 {
            println!("\n  {}", "Release is NOT ready.".red().bold());
        } else if warns > 0 {
            println!("\n  {}", "Release is ready (with warnings).".yellow().bold());
        } else {
            println!("\n  {}", "Release is ready!".green().bold());
        }
        println!();
    }
}
