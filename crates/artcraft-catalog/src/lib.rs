use artcraft_domain::{
    Application, ApplicationId, CatalogSource, CpuArch, Platform, VerificationStatus,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct RawApp {
    id: String,
    display_name: String,
    description: String,
    #[serde(default)]
    short_description: Option<String>,
    #[serde(default)]
    latest_version: Option<String>,
    publisher: String,
    #[serde(default)]
    official_repository: Option<String>,
    #[serde(default)]
    supported_platforms: Vec<String>,
    #[serde(default)]
    supported_arches: Vec<String>,
    #[serde(default)]
    known_package_ids: Vec<String>,
    #[serde(default)]
    executable_names: Vec<String>,
    #[serde(default)]
    desktop_ids: Vec<String>,
    #[serde(default)]
    known_data_locations: Vec<String>,
    #[serde(default)]
    is_community: bool,
}

#[derive(Debug, Deserialize)]
struct RawCatalog {
    applications: Vec<RawApp>,
}

fn parse_platform(s: &str) -> Platform {
    match s.to_lowercase().as_str() {
        "linux" => Platform::Linux,
        "macos" | "mac" => Platform::MacOS,
        "windows" | "win" => Platform::Windows,
        _ => Platform::Unknown,
    }
}

fn parse_arch(s: &str) -> CpuArch {
    match s.to_lowercase().as_str() {
        "x86_64" | "amd64" => CpuArch::X86_64,
        "aarch64" | "arm64" => CpuArch::Aarch64,
        "riscv64" => CpuArch::Riscv64,
        _ => CpuArch::Unknown,
    }
}

fn expand_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(s)
}

pub fn load_str(src: &str) -> Result<Vec<Application>, String> {
    let raw: RawCatalog = toml::from_str(src).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for a in raw.applications {
        out.push(Application {
            id: ApplicationId::new(a.id),
            display_name: a.display_name,
            description: a.description.clone(),
            short_description: a
                .short_description
                .clone()
                .unwrap_or_else(|| truncate_words(&a.description, 15)),
            latest_version: a.latest_version.clone(),
            publisher: a.publisher,
            official_repository: a.official_repository,
            supported_platforms: a
                .supported_platforms
                .iter()
                .map(|s| parse_platform(s))
                .collect(),
            supported_arches: a.supported_arches.iter().map(|s| parse_arch(s)).collect(),
            known_package_ids: a.known_package_ids,
            known_data_locations: a
                .known_data_locations
                .iter()
                .map(|s| expand_path(s))
                .collect(),
            executable_names: a.executable_names,
            desktop_ids: a.desktop_ids,
            catalog_source: CatalogSource::Bundled,
            verification_status: VerificationStatus::Unverified,
            is_community: a.is_community,
        });
    }
    Ok(out)
}

fn truncate_words(s: &str, n: usize) -> String {
    s.split_whitespace().take(n).collect::<Vec<_>>().join(" ")
}

pub fn bundled() -> Vec<Application> {
    const SRC: &str = include_str!("../../../catalog/artcraft.toml");
    load_str(SRC).expect("bundled catalog must parse")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_parses() {
        let apps = bundled();
        assert!(apps.len() >= 19);
    }
}
