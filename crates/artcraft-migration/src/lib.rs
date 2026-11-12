use artcraft_domain::{Installation, InstallationMethod, MigrationRecord};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("backup failed: {0}")]
    Backup(String),
    #[error("target unavailable: {0}")]
    TargetUnavailable(String),
    #[error("interrupted: {0}")]
    Interrupted(String),
}

pub struct MigrationPlanner;

impl MigrationPlanner {
    pub fn plan_steps() -> Vec<&'static str> {
        vec![
            "discover installations",
            "identify active processes",
            "verify target package availability",
            "verify architecture compatibility",
            "verify disk capacity",
            "discover persistent data",
            "classify data and uncertainty",
            "construct migration manifest",
            "present manifest",
            "create backup",
            "verify backup",
            "install target",
            "restore/transform state",
            "verify target installation",
            "verify migrated state",
            "preserve old installation",
            "record result",
        ]
    }
}

pub fn new_record(
    id: &str,
    app_id: artcraft_domain::ApplicationId,
    source: &Installation,
    target: InstallationMethod,
) -> MigrationRecord {
    MigrationRecord {
        id: id.to_string(),
        app_id,
        source_method: source.method.clone(),
        target_method: target,
        backup_path: None,
        status: "planned".into(),
        created_at: chrono_now(),
    }
}

fn chrono_now() -> String {
    // avoid chrono dep; use system time
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}
