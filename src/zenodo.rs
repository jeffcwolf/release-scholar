use crate::metadata::zenodo::ZenodoDeposit;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::path::Path;

const ZENODO_API: &str = "https://zenodo.org/api";
const ZENODO_SANDBOX_API: &str = "https://sandbox.zenodo.org/api";

pub struct ZenodoClient {
    client: Client,
    base_url: String,
    token: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositionResponse {
    pub id: u64,
    pub links: DepositionLinks,
    pub metadata: Option<serde_json::Value>,
    pub doi: Option<String>,
    pub conceptrecid: Option<String>,
    pub doi_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DepositionLinks {
    pub html: Option<String>,
    pub bucket: Option<String>,
    pub publish: Option<String>,
    #[serde(rename = "self")]
    pub self_link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileResponse {
    pub key: String,
    pub size: u64,
    pub checksum: String,
}

impl ZenodoClient {
    pub fn new(sandbox: bool) -> Result<Self, String> {
        let token = load_token(sandbox)?;
        let base_url = if sandbox {
            ZENODO_SANDBOX_API
        } else {
            ZENODO_API
        }
        .to_string();

        let client = Client::builder()
            .user_agent(format!("release-scholar/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| format!("Cannot create HTTP client: {}", e))?;
        Ok(ZenodoClient {
            client,
            base_url,
            token,
        })
    }

    /// Create a new empty deposition
    pub fn create_deposition(&self) -> Result<DepositionResponse, String> {
        let url = format!("{}/deposit/depositions", self.base_url);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .map_err(|e| format!("HTTP error creating deposition: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!(
                "Zenodo API error {} creating deposition: {}",
                status, body
            ));
        }

        resp.json::<DepositionResponse>()
            .map_err(|e| format!("Cannot parse deposition response: {}", e))
    }

    /// Upload a file to a deposition's bucket
    pub fn upload_file(
        &self,
        bucket_url: &str,
        file_path: &Path,
        filename: &str,
    ) -> Result<FileResponse, String> {
        let data =
            std::fs::read(file_path).map_err(|e| format!("Cannot read {}: {}", file_path.display(), e))?;

        let url = format!("{}/{}", bucket_url, filename);
        let resp = self
            .client
            .put(&url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .map_err(|e| format!("HTTP error uploading file: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!("Zenodo API error {} uploading: {}", status, body));
        }

        resp.json::<FileResponse>()
            .map_err(|e| format!("Cannot parse upload response: {}", e))
    }

    /// Update deposition metadata
    pub fn update_metadata(
        &self,
        deposition_id: u64,
        deposit: &ZenodoDeposit,
    ) -> Result<DepositionResponse, String> {
        let url = format!("{}/deposit/depositions/{}", self.base_url, deposition_id);
        let resp = self
            .client
            .put(&url)
            .bearer_auth(&self.token)
            .header("Content-Type", "application/json")
            .json(deposit)
            .send()
            .map_err(|e| format!("HTTP error updating metadata: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!(
                "Zenodo API error {} updating metadata: {}",
                status, body
            ));
        }

        resp.json::<DepositionResponse>()
            .map_err(|e| format!("Cannot parse metadata response: {}", e))
    }

    /// Publish the deposition (makes it permanent!)
    pub fn publish(&self, deposition_id: u64) -> Result<DepositionResponse, String> {
        let url = format!(
            "{}/deposit/depositions/{}/actions/publish",
            self.base_url, deposition_id
        );
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .send()
            .map_err(|e| format!("HTTP error publishing: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(format!(
                "Zenodo API error {} publishing: {}",
                status, body
            ));
        }

        resp.json::<DepositionResponse>()
            .map_err(|e| format!("Cannot parse publish response: {}", e))
    }

    pub fn base_web_url(&self) -> &str {
        if self.base_url.contains("sandbox") {
            "https://sandbox.zenodo.org"
        } else {
            "https://zenodo.org"
        }
    }
}

fn load_token(sandbox: bool) -> Result<String, String> {
    // Try environment variable first
    let env_var = if sandbox {
        "ZENODO_SANDBOX_TOKEN"
    } else {
        "ZENODO_TOKEN"
    };

    if let Ok(token) = std::env::var(env_var) {
        if !token.is_empty() {
            return Ok(token.trim().to_string());
        }
    }

    // Try config file
    let filename = if sandbox {
        "sandbox-token"
    } else {
        "token"
    };

    let config_dir = dirs::config_dir()
        .ok_or("Cannot determine config directory")?
        .join("release-scholar");
    let token_path = config_dir.join(filename);

    if token_path.exists() {
        let token = std::fs::read_to_string(&token_path)
            .map_err(|e| format!("Cannot read token from {}: {}", token_path.display(), e))?;
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    Err(format!(
        "No Zenodo token found. Set {} or save to {}",
        env_var,
        config_dir.join(filename).display()
    ))
}
