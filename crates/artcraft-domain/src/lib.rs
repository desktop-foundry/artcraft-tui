//! Domain model for artcraft-tui.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplicationId(pub String);

impl ApplicationId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Publisher(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuArch {
    X86_64,
    Aarch64,
    Riscv64,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationMethod {
    AptRepository,
    DebianPackage,
    DnfRepository,
    RpmPackage,
    ZypperRepository,
    PacmanRepository,
    ArchPackage,
    FlatpakUser,
    FlatpakSystem,
    AppImage,
    Tarball,
    WindowsMsi,
    WindowsPortable,
    MacOsDmg,
    MacOsApplicationBundle,
    HomebrewCask,
    LauncherSandbox,
    Unknown,
}

impl InstallationMethod {
    pub fn is_package_managed(&self) -> bool {
        matches!(
            self,
            Self::AptRepository
                | Self::DebianPackage
                | Self::DnfRepository
                | Self::RpmPackage
                | Self::ZypperRepository
                | Self::PacmanRepository
                | Self::ArchPackage
                | Self::FlatpakUser
                | Self::FlatpakSystem
                | Self::HomebrewCask
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallationScope {
    System,
    User,
    Portable,
    Sandbox,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Outdated,
    Broken,
    Duplicate,
    NonpreferredMethod,
    Orphaned,
    PartiallyInstalled,
    Unknown,
    NotInstalled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionConfidence {
    High,
    Medium,
    Low,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Verified,
    Unverified,
    Suspicious,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatalogSource {
    Bundled,
    Refreshed,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: ApplicationId,
    pub display_name: String,
    pub description: String,
    pub publisher: String,
    pub official_repository: Option<String>,
    pub supported_platforms: Vec<Platform>,
    pub supported_arches: Vec<CpuArch>,
    pub known_package_ids: Vec<String>,
    pub known_data_locations: Vec<PathBuf>,
    pub executable_names: Vec<String>,
    pub desktop_ids: Vec<String>,
    pub catalog_source: CatalogSource,
    pub verification_status: VerificationStatus,
    pub is_community: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installation {
    pub id: String,
    pub app_id: ApplicationId,
    pub installed_version: Option<String>,
    pub method: InstallationMethod,
    pub scope: InstallationScope,
    pub executable_path: Option<PathBuf>,
    pub package_identifier: Option<String>,
    pub package_manager: Option<String>,
    pub architecture: CpuArch,
    pub installation_root: Option<PathBuf>,
    pub provenance: Option<String>,
    pub health: HealthStatus,
    pub confidence: DetectionConfidence,
    pub last_verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredState {
    pub preferred_method: InstallationMethod,
    pub desired_apps: Vec<ApplicationId>,
    pub version_constraints: std::collections::BTreeMap<String, String>,
    pub scope: InstallationScope,
    pub update_policy: String,
    pub backup_policy: String,
    pub duplicate_policy: String,
    pub data_retention_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFinding {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub evidence: Vec<String>,
    pub confidence: DetectionConfidence,
    pub recommended_action: Option<String>,
    pub auto_repair_supported: bool,
    pub verified_fact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationOperation {
    pub id: String,
    pub kind: String,
    pub app_id: Option<ApplicationId>,
    pub from: Option<InstallationMethod>,
    pub to: Option<InstallationMethod>,
    pub requires_privileges: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationPlan {
    pub id: String,
    pub operations: Vec<ReconciliationOperation>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub id: String,
    pub app_id: ApplicationId,
    pub source_method: InstallationMethod,
    pub target_method: InstallationMethod,
    pub backup_path: Option<PathBuf>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub created_at: String,
    pub source_installation: String,
    pub source_method: InstallationMethod,
    pub target_method: InstallationMethod,
    pub entries: Vec<BackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: Platform,
    pub distro_id: String,
    pub distro_like: Vec<String>,
    pub version: String,
    pub arch: CpuArch,
    pub package_managers: Vec<String>,
    pub flatpak_available: bool,
    pub is_root: bool,
    pub xdg_data_home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub available_disk_bytes: Option<u64>,
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self {
            os: Platform::Unknown,
            distro_id: String::new(),
            distro_like: vec![],
            version: String::new(),
            arch: CpuArch::Unknown,
            package_managers: vec![],
            flatpak_available: false,
            is_root: false,
            xdg_data_home: None,
            xdg_config_home: None,
            available_disk_bytes: None,
        }
    }
}
