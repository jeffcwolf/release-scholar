use crate::metadata::citation::CitationCff;
use crate::metadata::zenodo::ZenodoDeposit;
use crate::zenodo::ZenodoClient;
use colored::Colorize;
use std::path::Path;

pub fn run(project_dir: &Path, sandbox: bool, confirm: bool) -> Result<(), String> {
    let project_dir = std::fs::canonicalize(project_dir)
        .map_err(|e| format!("Invalid project directory: {}", e))?;

    // Determine version from git tag
    let version = get_version(&project_dir)?;
    let tag = format!("v{}", version);

    let config = crate::config::Config::load(&project_dir);
    let release_dir = project_dir.join(&config.archive_dir).join(&tag);

    if !release_dir.exists() {
        return Err(format!(
            "Release bundle not found at {}. Run `release-scholar build` first.",
            release_dir.display()
        ));
    }

    // Find the archive file
    let archive_path = find_archive(&release_dir)?;
    let archive_name = archive_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Load citation metadata
    let citation_path = project_dir.join("CITATION.cff");
    let cff = CitationCff::from_file(&citation_path)?;
    let deposit = ZenodoDeposit::from_citation(&cff, &config);

    let env_label = if sandbox {
        "SANDBOX".yellow().bold()
    } else {
        "PRODUCTION".red().bold()
    };
    println!(
        "\n{} Publishing {} to Zenodo [{}]...\n",
        ">>>".bold(),
        tag.bold(),
        env_label
    );

    // Connect to Zenodo
    let client = ZenodoClient::new(sandbox)?;

    // Step 1: Create deposition
    print!("  Creating deposition... ");
    let deposition = client.create_deposition()?;
    let deposition_id = deposition.id;
    let bucket_url = deposition
        .links
        .bucket
        .ok_or("No bucket URL in deposition response")?;
    println!("{} (id: {})", "done".green(), deposition_id);

    // Step 2: Upload archive
    print!("  Uploading {}... ", archive_name);
    let file_resp = client.upload_file(&bucket_url, &archive_path, &archive_name)?;
    println!(
        "{} ({} bytes, checksum: {})",
        "done".green(),
        file_resp.size,
        file_resp.checksum
    );

    // Step 3: Update metadata
    print!("  Setting metadata... ");
    client.update_metadata(deposition_id, &deposit)?;
    println!("{}", "done".green());

    // Step 4: Publish or leave as draft
    let web_url = format!(
        "{}/deposit/{}",
        client.base_web_url(),
        deposition_id
    );

    if confirm {
        print!("  Publishing... ");
        client.publish(deposition_id)?;
        println!("{}", "done".green());
        println!(
            "\n  {} Deposit published!",
            "OK".green().bold()
        );
        println!("  View at: {}", web_url);
    } else {
        println!(
            "\n  {} Draft deposit created (not yet published).",
            "OK".green().bold()
        );
        println!("  Review at: {}", web_url);
        println!(
            "\n  To publish, run: release-scholar publish --project-dir {} --confirm",
            project_dir.display()
        );
    }

    println!();
    Ok(())
}

fn get_version(project_dir: &Path) -> Result<String, String> {
    let repo =
        git2::Repository::open(project_dir).map_err(|e| format!("Cannot open repo: {}", e))?;
    let head = repo.head().map_err(|e| format!("Cannot read HEAD: {}", e))?;
    let head_oid = head.target().ok_or("HEAD has no target")?;

    let tag_names = repo.tag_names(None).map_err(|e| e.to_string())?;
    let semver_re = regex::Regex::new(r"^v(\d+\.\d+\.\d+)$").unwrap();

    for i in 0..tag_names.len() {
        let name = match tag_names.get(i) {
            Some(n) => n,
            None => continue,
        };
        if let Some(caps) = semver_re.captures(name) {
            let tag_oid = match repo.revparse_single(&format!("refs/tags/{}", name)) {
                Ok(obj) => obj.peel_to_commit().map(|c| c.id()).unwrap_or(obj.id()),
                Err(_) => continue,
            };
            if tag_oid == head_oid {
                return Ok(caps[1].to_string());
            }
        }
    }

    Err("HEAD has no semver tag (vX.Y.Z)".to_string())
}

fn find_archive(release_dir: &Path) -> Result<std::path::PathBuf, String> {
    for entry in std::fs::read_dir(release_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".tar.gz") {
                return Ok(path);
            }
        }
    }
    Err(format!(
        "No .tar.gz archive found in {}",
        release_dir.display()
    ))
}
