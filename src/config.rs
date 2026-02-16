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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorConfig {
    pub name: Option<String>,
    pub orcid: Option<String>,
    pub email: Option<String>,
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
        }
    }
}

impl Config {
    pub fn load(project_dir: &Path) -> Self {
        let config_path = project_dir.join(".release-scholar.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }
}
