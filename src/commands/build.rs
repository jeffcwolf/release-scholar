use crate::archive::{checksum, tarball};
use crate::config::Config;
use crate::metadata::citation::CitationCff;
use crate::metadata::zenodo::ZenodoDeposit;
use colored::Colorize;
use std::path::Path;

pub fn run(project_dir: &Path) -> Result<(), String> {
    let project_dir = std::fs::canonicalize(project_dir)
        .map_err(|e| format!("Invalid project directory: {}", e))?;
    let config = Config::load(&project_dir);

    // Determine version from git tag
    let version = get_version_from_tag(&project_dir)?;
    let tag = format!("v{}", version);

    println!("{}", format!("Building release bundle for {}...", tag).bold());
    println!();

    // Create output directory
    let release_dir = project_dir.join(&config.archive_dir).join(&tag);
    std::fs::create_dir_all(&release_dir)
        .map_err(|e| format!("Cannot create release directory: {}", e))?;

    // Create archive
    let project_name = project_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let archive_name = format!("{}-{}.tar.gz", project_name, tag);
    let archive_path = release_dir.join(&archive_name);

    print!("  Creating archive... ");
    tarball::create_archive(&project_dir, &tag, &archive_path)?;
    println!("{}", "done".green());

    // Generate checksum
    print!("  Generating checksum... ");
    let hash = checksum::sha256_file(&archive_path)?;
    let checksums_path = release_dir.join("checksums.txt");
    std::fs::write(&checksums_path, format!("{}  {}\n", hash, archive_name))
        .map_err(|e| format!("Cannot write checksums: {}", e))?;
    println!("{}", "done".green());

    // Generate Zenodo metadata from CITATION.cff
    let citation_path = project_dir.join("CITATION.cff");
    if citation_path.exists() {
        print!("  Generating metadata.json... ");
        let cff = CitationCff::from_file(&citation_path)?;
        let zenodo = ZenodoDeposit::from_citation(&cff, &config);
        let metadata_path = release_dir.join("metadata.json");
        std::fs::write(&metadata_path, zenodo.to_json())
            .map_err(|e| format!("Cannot write metadata.json: {}", e))?;
        println!("{}", "done".green());

        // Copy CITATION.cff into bundle
        let cff_dest = release_dir.join("CITATION.cff");
        std::fs::copy(&citation_path, &cff_dest)
            .map_err(|e| format!("Cannot copy CITATION.cff: {}", e))?;
    }

    // Copy codemeta.json if it exists
    let codemeta_path = project_dir.join("codemeta.json");
    if codemeta_path.exists() {
        std::fs::copy(&codemeta_path, release_dir.join("codemeta.json"))
            .map_err(|e| format!("Cannot copy codemeta.json: {}", e))?;
        println!("  {} codemeta.json", "Copied".green());
    }

    println!();
    println!(
        "  {} Release bundle: {}",
        "OK".green().bold(),
        release_dir.display()
    );
    println!("  Archive:   {}", archive_name);
    println!("  SHA256:    {}", hash);
    println!();

    Ok(())
}

fn get_version_from_tag(project_dir: &Path) -> Result<String, String> {
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

    Err("HEAD has no semver tag (vX.Y.Z). Run `release-scholar check` first.".to_string())
}
