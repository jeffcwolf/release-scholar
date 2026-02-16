use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Forge {
    Codeberg,
    Github,
    Gitlab,
}

impl Default for Forge {
    fn default() -> Self {
        Forge::Codeberg
    }
}

impl std::fmt::Display for Forge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Forge::Codeberg => write!(f, "codeberg"),
            Forge::Github => write!(f, "github"),
            Forge::Gitlab => write!(f, "gitlab"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorConfig {
    pub name: Option<String>,
    pub orcid: Option<String>,
    pub email: Option<String>,
}

impl AuthorConfig {
    /// Merge: self takes priority, fallback fills in gaps
    fn merge_with_fallback(&mut self, fallback: &AuthorConfig) {
        if self.name.is_none() {
            self.name = fallback.name.clone();
        }
        if self.orcid.is_none() {
            self.orcid = fallback.orcid.clone();
        }
        if self.email.is_none() {
            self.email = fallback.email.clone();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub forge: Forge,
    pub forge_url: Option<String>,
    #[serde(default = "default_required_files")]
    pub required_files: Vec<String>,
    #[serde(default = "default_archive_dir")]
    pub archive_dir: String,
    #[serde(default = "default_language")]
    pub language: String,
    pub author: Option<AuthorConfig>,
    pub mirrors: Option<MirrorsConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MirrorsConfig {
    pub codeberg_user: Option<String>,
    pub codeberg_token: Option<String>,
    pub github_user: Option<String>,
    pub github_token: Option<String>,
    pub gitlab_user: Option<String>,
    pub gitlab_token: Option<String>,
}

fn default_language() -> String {
    "eng".to_string()
}

fn default_required_files() -> Vec<String> {
    vec![
        "LICENSE".to_string(),
        "README.md".to_string(),
        "CHANGELOG.md".to_string(),
        "CITATION.cff".to_string(),
    ]
}

fn default_archive_dir() -> String {
    "release".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            forge: Forge::default(),
            forge_url: None,
            required_files: default_required_files(),
            archive_dir: default_archive_dir(),
            language: default_language(),
            author: None,
            mirrors: None,
        }
    }
}

impl Config {
    /// Load config: global defaults ← project overrides.
    /// Author info merges (project fields override global fields).
    pub fn load(project_dir: &Path) -> Self {
        let global = load_global_config();
        let project_path = project_dir.join(".release-scholar.toml");

        let mut config = if project_path.exists() {
            let content = std::fs::read_to_string(&project_path).unwrap_or_default();
            toml::from_str::<Config>(&content).unwrap_or_default()
        } else {
            Config::default()
        };

        // Merge author: project author takes priority, global fills gaps
        if let Some(global_author) = &global.author {
            match &mut config.author {
                Some(project_author) => {
                    project_author.merge_with_fallback(global_author);
                }
                None => {
                    config.author = Some(global_author.clone());
                }
            }
        }

        // Merge mirrors: global provides defaults
        if config.mirrors.is_none() {
            config.mirrors = global.mirrors;
        }

        config
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    /// Path to the global config file
    pub fn global_config_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("release-scholar").join("config.toml"))
    }
}

/// Load global config from ~/.config/release-scholar/config.toml
/// (or ~/Library/Application Support/release-scholar/config.toml on macOS)
fn load_global_config() -> Config {
    let path = match Config::global_config_path() {
        Some(p) => p,
        None => return Config::default(),
    };
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}
