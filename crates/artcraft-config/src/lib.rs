use artcraft_domain::{DesiredState, InstallationMethod, InstallationScope};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("serialize: {0}")]
    Ser(#[from] toml::ser::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub preferred_method: Option<String>,
    pub desired_apps: Vec<String>,
    pub scope: Option<String>,
}

impl AppConfig {
    pub fn path() -> PathBuf {
        let base = directories::ProjectDirs::from("com", "artcraft", "artcraft-tui")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("config.toml")
    }

    pub fn load() -> Result<Self, ConfigError> {
        let p = Self::path();
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(p)?;
        Ok(toml::from_str(&s)?)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(p, s)?;
        Ok(())
    }

    pub fn to_desired_state(&self) -> DesiredState {
        let method = self
            .preferred_method
            .as_deref()
            .and_then(parse_method)
            .unwrap_or(InstallationMethod::Unknown);
        let apps = self
            .desired_apps
            .iter()
            .map(|s| artcraft_domain::ApplicationId::new(s.clone()))
            .collect();
        DesiredState {
            preferred_method: method,
            desired_apps: apps,
            version_constraints: Default::default(),
            scope: InstallationScope::User,
            update_policy: "manual".into(),
            backup_policy: "always".into(),
            duplicate_policy: "resolve".into(),
            data_retention_policy: "preserve".into(),
        }
    }
}

fn parse_method(s: &str) -> Option<InstallationMethod> {
    match s {
        "apt" | "AptRepository" => Some(InstallationMethod::AptRepository),
        "deb" | "DebianPackage" => Some(InstallationMethod::DebianPackage),
        "flatpak" | "FlatpakUser" => Some(InstallationMethod::FlatpakUser),
        "appimage" | "AppImage" => Some(InstallationMethod::AppImage),
        "tarball" | "Tarball" => Some(InstallationMethod::Tarball),
        _ => None,
    }
}
