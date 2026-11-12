use artcraft_domain::{BackupEntry, BackupManifest, InstallationMethod};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tar::Builder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("integrity: {0}")]
    Integrity(String),
}

pub fn create_backup(
    source_dir: &Path,
    dest_archive: &Path,
    manifest: &mut BackupManifest,
) -> Result<(), BackupError> {
    if let Some(parent) = dest_archive.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(dest_archive)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(enc);

    // walk source_dir
    for entry in walkdir(source_dir)? {
        let rel = entry.strip_prefix(source_dir).unwrap_or(&entry);
        if entry.is_dir() {
            builder.append_dir(rel, &entry)?;
        } else {
            let mut f = File::open(&entry)?;
            builder.append_file(rel, &mut f)?;
            let meta = fs::metadata(&entry)?;
            let sha = artcraft_integrity::sha256_file(&entry)
                .map_err(|e| BackupError::Integrity(e.to_string()))?;
            manifest.entries.push(BackupEntry {
                path: rel.to_path_buf(),
                size: meta.len(),
                sha256: sha,
            });
        }
    }
    builder.finish()?;
    Ok(())
}

fn walkdir(root: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut out = Vec::new();
    if !root.exists() {
        return Ok(out);
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        out.push(dir.clone());
        if dir.is_dir() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                stack.push(entry.path());
            }
        }
    }
    Ok(out)
}

pub fn verify_backup(archive: &Path, manifest: &BackupManifest) -> Result<bool, BackupError> {
    // Re-extract to temp and verify checksums
    let tmp = tempfile::tempdir().map_err(io::Error::other)?;
    let file = File::open(archive)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(tmp.path())?;
    for e in &manifest.entries {
        let p = tmp.path().join(&e.path);
        if !p.exists() {
            return Ok(false);
        }
        let sha = artcraft_integrity::sha256_file(&p)
            .map_err(|err| BackupError::Integrity(err.to_string()))?;
        if sha != e.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn new_manifest(
    id: &str,
    source_installation: &str,
    source_method: InstallationMethod,
    target_method: InstallationMethod,
) -> BackupManifest {
    BackupManifest {
        id: id.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        source_installation: source_installation.to_string(),
        source_method,
        target_method,
        entries: vec![],
    }
}
