use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("mismatch: expected {expected}, got {actual}")]
    Mismatch { expected: String, actual: String },
}

pub fn sha256_file(path: &std::path::Path) -> Result<String, IntegrityError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_file(path: &std::path::Path, expected: &str) -> Result<(), IntegrityError> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(IntegrityError::Mismatch {
            expected: expected.to_string(),
            actual,
        })
    }
}
