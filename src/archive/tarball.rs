use flate2::write::GzEncoder;
use flate2::Compression;
use git2::Repository;
use std::path::Path;
use tar::Header;

pub fn create_archive(project_dir: &Path, tag: &str, output_path: &Path) -> Result<(), String> {
    let repo = Repository::open(project_dir).map_err(|e| format!("Cannot open repo: {}", e))?;

    // Resolve tag to tree
    let obj = repo
        .revparse_single(&format!("refs/tags/{}", tag))
        .map_err(|e| format!("Cannot find tag {}: {}", tag, e))?;
    let commit = obj
        .peel_to_commit()
        .map_err(|e| format!("Cannot peel to commit: {}", e))?;
    let tree = commit
        .tree()
        .map_err(|e| format!("Cannot get tree: {}", e))?;

    let file =
        std::fs::File::create(output_path).map_err(|e| format!("Cannot create archive: {}", e))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut ar = tar::Builder::new(enc);

    let prefix = format!(
        "{}-{}",
        project_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy(),
        tag
    );

    // Collect all blobs sorted by path for determinism
    let mut entries: Vec<(String, Vec<u8>, u32)> = Vec::new();
    collect_tree_entries(&repo, &tree, "", &mut entries)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let fixed_mtime = commit.time().seconds() as u64;

    for (path, data, mode) in &entries {
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mtime(fixed_mtime);
        // Map git mode to tar mode
        let tar_mode = if *mode == 0o100755 { 0o755 } else { 0o644 };
        header.set_mode(tar_mode);
        header.set_uid(0);
        header.set_gid(0);
        header.set_username("root").ok();
        header.set_groupname("root").ok();
        header.set_cksum();

        let full_path = format!("{}/{}", prefix, path);
        ar.append_data(&mut header, &full_path, data.as_slice())
            .map_err(|e| format!("Cannot add {}: {}", path, e))?;
    }

    let enc = ar
        .into_inner()
        .map_err(|e| format!("Cannot finalize tar: {}", e))?;
    enc.finish()
        .map_err(|e| format!("Cannot finalize gzip: {}", e))?;

    Ok(())
}

fn collect_tree_entries(
    repo: &Repository,
    tree: &git2::Tree,
    prefix: &str,
    entries: &mut Vec<(String, Vec<u8>, u32)>,
) -> Result<(), String> {
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("").to_string();
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };

        match entry.kind() {
            Some(git2::ObjectType::Blob) => {
                let blob = repo
                    .find_blob(entry.id())
                    .map_err(|e| format!("Cannot read blob {}: {}", path, e))?;
                entries.push((path, blob.content().to_vec(), entry.filemode() as u32));
            }
            Some(git2::ObjectType::Tree) => {
                let subtree = repo
                    .find_tree(entry.id())
                    .map_err(|e| format!("Cannot read tree {}: {}", path, e))?;
                collect_tree_entries(repo, &subtree, &path, entries)?;
            }
            _ => {}
        }
    }
    Ok(())
}
