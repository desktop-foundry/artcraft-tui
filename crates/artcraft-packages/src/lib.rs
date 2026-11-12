use artcraft_domain::{Application, Installation, InstallationMethod};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("unsupported: {0}")]
    Unsupported(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("command failed: {0}")]
    CommandFailed(String),
}

#[derive(Debug, Clone)]
pub struct PackageCapabilities {
    pub can_discover: bool,
    pub can_install: bool,
    pub can_remove: bool,
    pub requires_privileges: bool,
    pub supported_methods: Vec<InstallationMethod>,
}

#[derive(Debug, Clone)]
pub struct PackageCandidate {
    pub app_id: artcraft_domain::ApplicationId,
    pub method: InstallationMethod,
    pub package_identifier: String,
    pub version: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub method: InstallationMethod,
    pub commands: Vec<Vec<String>>,
    pub requires_privileges: bool,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RemovePlan {
    pub installation_id: String,
    pub commands: Vec<Vec<String>>,
    pub requires_privileges: bool,
    pub description: String,
}

pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> PackageCapabilities;
    fn discover(&self, apps: &[Application]) -> Result<Vec<Installation>, PackageError>;
    fn resolve(&self, app: &Application) -> Result<PackageCandidate, PackageError>;
    fn plan_install(&self, candidate: &PackageCandidate) -> Result<InstallPlan, PackageError>;
    fn plan_remove(&self, installation: &Installation) -> Result<RemovePlan, PackageError>;
}

pub struct AptManager;
impl PackageManager for AptManager {
    fn name(&self) -> &str {
        "apt"
    }
    fn capabilities(&self) -> PackageCapabilities {
        PackageCapabilities {
            can_discover: true,
            can_install: true,
            can_remove: true,
            requires_privileges: true,
            supported_methods: vec![
                InstallationMethod::AptRepository,
                InstallationMethod::DebianPackage,
            ],
        }
    }
    fn discover(&self, _apps: &[Application]) -> Result<Vec<Installation>, PackageError> {
        Ok(vec![])
    }
    fn resolve(&self, app: &Application) -> Result<PackageCandidate, PackageError> {
        let pkg = app
            .known_package_ids
            .first()
            .cloned()
            .ok_or_else(|| PackageError::Unsupported(app.id.to_string()))?;
        Ok(PackageCandidate {
            app_id: app.id.clone(),
            method: InstallationMethod::AptRepository,
            package_identifier: pkg,
            version: None,
            source_url: None,
        })
    }
    fn plan_install(&self, c: &PackageCandidate) -> Result<InstallPlan, PackageError> {
        Ok(InstallPlan {
            method: c.method.clone(),
            commands: vec![vec![
                "apt-get".into(),
                "install".into(),
                "-y".into(),
                c.package_identifier.clone(),
            ]],
            requires_privileges: true,
            description: format!("Install {} via apt", c.package_identifier),
        })
    }
    fn plan_remove(&self, i: &Installation) -> Result<RemovePlan, PackageError> {
        Ok(RemovePlan {
            installation_id: i.id.clone(),
            commands: vec![vec![
                "apt-get".into(),
                "remove".into(),
                "-y".into(),
                i.package_identifier.clone().unwrap_or_default(),
            ]],
            requires_privileges: true,
            description: format!("Remove {}", i.id),
        })
    }
}

pub struct FlatpakManager;
impl PackageManager for FlatpakManager {
    fn name(&self) -> &str {
        "flatpak"
    }
    fn capabilities(&self) -> PackageCapabilities {
        PackageCapabilities {
            can_discover: true,
            can_install: true,
            can_remove: true,
            requires_privileges: false,
            supported_methods: vec![
                InstallationMethod::FlatpakUser,
                InstallationMethod::FlatpakSystem,
            ],
        }
    }
    fn discover(&self, _apps: &[Application]) -> Result<Vec<Installation>, PackageError> {
        Ok(vec![])
    }
    fn resolve(&self, app: &Application) -> Result<PackageCandidate, PackageError> {
        let pkg = app
            .known_package_ids
            .first()
            .cloned()
            .ok_or_else(|| PackageError::Unsupported(app.id.to_string()))?;
        Ok(PackageCandidate {
            app_id: app.id.clone(),
            method: InstallationMethod::FlatpakUser,
            package_identifier: pkg,
            version: None,
            source_url: None,
        })
    }
    fn plan_install(&self, c: &PackageCandidate) -> Result<InstallPlan, PackageError> {
        Ok(InstallPlan {
            method: c.method.clone(),
            commands: vec![vec![
                "flatpak".into(),
                "install".into(),
                "--user".into(),
                "-y".into(),
                "artcraft-oci".into(),
                c.package_identifier.clone(),
            ]],
            requires_privileges: false,
            description: format!("Install {} via flatpak", c.package_identifier),
        })
    }
    fn plan_remove(&self, i: &Installation) -> Result<RemovePlan, PackageError> {
        Ok(RemovePlan {
            installation_id: i.id.clone(),
            commands: vec![vec![
                "flatpak".into(),
                "uninstall".into(),
                "-y".into(),
                i.package_identifier.clone().unwrap_or_default(),
            ]],
            requires_privileges: false,
            description: format!("Remove {}", i.id),
        })
    }
}

pub struct AppImageManager;
impl PackageManager for AppImageManager {
    fn name(&self) -> &str {
        "appimage"
    }
    fn capabilities(&self) -> PackageCapabilities {
        PackageCapabilities {
            can_discover: true,
            can_install: false,
            can_remove: false,
            requires_privileges: false,
            supported_methods: vec![InstallationMethod::AppImage],
        }
    }
    fn discover(&self, _apps: &[Application]) -> Result<Vec<Installation>, PackageError> {
        Ok(vec![])
    }
    fn resolve(&self, app: &Application) -> Result<PackageCandidate, PackageError> {
        Err(PackageError::Unsupported(app.id.to_string()))
    }
    fn plan_install(&self, _c: &PackageCandidate) -> Result<InstallPlan, PackageError> {
        Err(PackageError::Unsupported(
            "appimage install not planned".into(),
        ))
    }
    fn plan_remove(&self, _i: &Installation) -> Result<RemovePlan, PackageError> {
        Err(PackageError::Unsupported(
            "appimage remove not planned".into(),
        ))
    }
}
