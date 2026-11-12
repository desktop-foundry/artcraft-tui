use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    PlatformDetected {
        os: String,
    },
    ApplicationDiscovered {
        app: String,
    },
    InstallationDetected {
        installation_id: String,
        app_id: String,
    },
    DuplicateInstallationDetected {
        app_id: String,
    },
    DesiredStateChanged,
    DiagnosticCompleted {
        findings: usize,
    },
    ReconciliationPlanned {
        plan_id: String,
    },
    BackupCreated {
        backup_id: String,
    },
    BackupVerified {
        backup_id: String,
    },
    MigrationStarted {
        migration_id: String,
    },
    InstallationCompleted {
        app_id: String,
    },
    InstallationVerified {
        app_id: String,
    },
    MigrationCompleted {
        migration_id: String,
    },
    MigrationFailed {
        migration_id: String,
        reason: String,
    },
    RollbackStarted {
        migration_id: String,
    },
    RollbackCompleted {
        migration_id: String,
    },
}

impl DomainEvent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::PlatformDetected { .. } => "PlatformDetected",
            Self::ApplicationDiscovered { .. } => "ApplicationDiscovered",
            Self::InstallationDetected { .. } => "InstallationDetected",
            Self::DuplicateInstallationDetected { .. } => "DuplicateInstallationDetected",
            Self::DesiredStateChanged => "DesiredStateChanged",
            Self::DiagnosticCompleted { .. } => "DiagnosticCompleted",
            Self::ReconciliationPlanned { .. } => "ReconciliationPlanned",
            Self::BackupCreated { .. } => "BackupCreated",
            Self::BackupVerified { .. } => "BackupVerified",
            Self::MigrationStarted { .. } => "MigrationStarted",
            Self::InstallationCompleted { .. } => "InstallationCompleted",
            Self::InstallationVerified { .. } => "InstallationVerified",
            Self::MigrationCompleted { .. } => "MigrationCompleted",
            Self::MigrationFailed { .. } => "MigrationFailed",
            Self::RollbackStarted { .. } => "RollbackStarted",
            Self::RollbackCompleted { .. } => "RollbackCompleted",
        }
    }
}

pub trait EventSink: Send + Sync {
    fn record(&self, event: DomainEvent);
}
