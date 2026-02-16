use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationCff {
    #[serde(rename = "cff-version")]
    pub cff_version: String,
    pub title: String,
    #[serde(rename = "type", default = "default_type")]
    pub cff_type: String,
    pub authors: Vec<CffAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(rename = "date-released", skip_serializing_if = "Option::is_none")]
    pub date_released: Option<String>,
    #[serde(rename = "repository-code", skip_serializing_if = "Option::is_none")]
    pub repository_code: Option<String>,
    #[serde(rename = "abstract", skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
}

fn default_type() -> String {
    "software".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CffAuthor {
    #[serde(rename = "family-names")]
    pub family_names: String,
    #[serde(rename = "given-names")]
    pub given_names: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
}

impl CitationCff {
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse CITATION.cff: {}", e))
    }
}
