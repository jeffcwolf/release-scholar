use crate::config::Config;
use crate::metadata::citation::CitationCff;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ZenodoDeposit {
    pub metadata: ZenodoMetadata,
}

#[derive(Debug, Serialize)]
pub struct ZenodoMetadata {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub creators: Vec<ZenodoCreator>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    pub upload_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_identifiers: Vec<ZenodoRelatedIdentifier>,
}

#[derive(Debug, Serialize)]
pub struct ZenodoCreator {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ZenodoRelatedIdentifier {
    pub identifier: String,
    pub relation: String,
    pub resource_type: Option<String>,
    pub scheme: String,
}

impl ZenodoDeposit {
    pub fn from_citation(cff: &CitationCff, config: &Config) -> Self {
        let creators = cff
            .authors
            .iter()
            .map(|a| {
                let orcid = a.orcid.as_ref().map(|o| {
                    o.strip_prefix("https://orcid.org/")
                        .unwrap_or(o)
                        .to_string()
                });
                ZenodoCreator {
                    name: format!("{}, {}", a.family_names, a.given_names),
                    orcid,
                    affiliation: a.affiliation.clone(),
                }
            })
            .collect();

        // Related identifiers — add repository URL if present
        let mut related_identifiers = Vec::new();
        if let Some(repo_url) = &cff.repository_code {
            related_identifiers.push(ZenodoRelatedIdentifier {
                identifier: repo_url.clone(),
                relation: "isSupplementTo".to_string(),
                resource_type: Some("software".to_string()),
                scheme: "url".to_string(),
            });
        }

        ZenodoDeposit {
            metadata: ZenodoMetadata {
                title: cff.title.clone(),
                description: cff.abstract_text.clone(),
                creators,
                keywords: cff.keywords.clone(),
                license: cff.license.clone(),
                version: cff.version.clone(),
                publication_date: cff.date_released.clone(),
                upload_type: "software".to_string(),
                language: Some(config.language.clone()),
                related_identifiers,
            },
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}
