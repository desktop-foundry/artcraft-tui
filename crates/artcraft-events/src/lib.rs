use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
    PreferredMethodSelected {
        method: String,
    },
    AppSelectionChanged {
        apps: Vec<String>,
    },
    ReconciliationApproved {
        plan_id: String,
    },
    ReconciliationExecuted {
        plan_id: String,
        operations: usize,
    },
    ReconciliationExecutionFailed {
        plan_id: String,
        reason: String,
    },
    OperationCompleted {
        operation_id: String,
        kind: String,
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
            Self::PreferredMethodSelected { .. } => "PreferredMethodSelected",
            Self::AppSelectionChanged { .. } => "AppSelectionChanged",
            Self::ReconciliationApproved { .. } => "ReconciliationApproved",
            Self::ReconciliationExecuted { .. } => "ReconciliationExecuted",
            Self::ReconciliationExecutionFailed { .. } => "ReconciliationExecutionFailed",
            Self::OperationCompleted { .. } => "OperationCompleted",
        }
    }

    /// JTD schema for this event (embedded).
    pub fn schema_json(&self) -> Option<&'static str> {
        match self.name() {
            "PlatformDetected" => Some(include_str!("../schemas/PlatformDetected.jtd.json")),
            "ApplicationDiscovered" => {
                Some(include_str!("../schemas/ApplicationDiscovered.jtd.json"))
            }
            "InstallationDetected" => {
                Some(include_str!("../schemas/InstallationDetected.jtd.json"))
            }
            "DuplicateInstallationDetected" => Some(include_str!(
                "../schemas/DuplicateInstallationDetected.jtd.json"
            )),
            "DesiredStateChanged" => Some(include_str!("../schemas/DesiredStateChanged.jtd.json")),
            "DiagnosticCompleted" => Some(include_str!("../schemas/DiagnosticCompleted.jtd.json")),
            "ReconciliationPlanned" => {
                Some(include_str!("../schemas/ReconciliationPlanned.jtd.json"))
            }
            "BackupCreated" => Some(include_str!("../schemas/BackupCreated.jtd.json")),
            "BackupVerified" => Some(include_str!("../schemas/BackupVerified.jtd.json")),
            "MigrationStarted" => Some(include_str!("../schemas/MigrationStarted.jtd.json")),
            "InstallationCompleted" => {
                Some(include_str!("../schemas/InstallationCompleted.jtd.json"))
            }
            "InstallationVerified" => {
                Some(include_str!("../schemas/InstallationVerified.jtd.json"))
            }
            "MigrationCompleted" => Some(include_str!("../schemas/MigrationCompleted.jtd.json")),
            "MigrationFailed" => Some(include_str!("../schemas/MigrationFailed.jtd.json")),
            "RollbackStarted" => Some(include_str!("../schemas/RollbackStarted.jtd.json")),
            "RollbackCompleted" => Some(include_str!("../schemas/RollbackCompleted.jtd.json")),
            "PreferredMethodSelected" => {
                Some(include_str!("../schemas/PreferredMethodSelected.jtd.json"))
            }
            "AppSelectionChanged" => Some(include_str!("../schemas/AppSelectionChanged.jtd.json")),
            "ReconciliationApproved" => {
                Some(include_str!("../schemas/ReconciliationApproved.jtd.json"))
            }
            "ReconciliationExecuted" => {
                Some(include_str!("../schemas/ReconciliationExecuted.jtd.json"))
            }
            "ReconciliationExecutionFailed" => Some(include_str!(
                "../schemas/ReconciliationExecutionFailed.jtd.json"
            )),
            "OperationCompleted" => Some(include_str!("../schemas/OperationCompleted.jtd.json")),
            _ => None,
        }
    }

    /// Validate this event's JSON serialization against its JTD schema.
    pub fn validate_self(&self) -> bool {
        let Ok(schema_str) = serde_json::to_string(&serde_json::to_value(self).unwrap_or_default())
        else {
            return false;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&schema_str) else {
            return false;
        };
        let Some(s) = self.schema_json() else {
            return true;
        };
        let Ok(serde_schema) = serde_json::from_str::<jtd::SerdeSchema>(s) else {
            return false;
        };
        let Ok(schema) = jtd::Schema::from_serde_schema(serde_schema) else {
            return false;
        };
        if schema.validate().is_err() {
            return false;
        }
        jtd::validate(&schema, &value, Default::default())
            .map(|errs| errs.is_empty())
            .unwrap_or(false)
    }
}

pub trait EventSink: Send + Sync {
    fn record(&self, event: DomainEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn jtd_validates_event() {
        let e = DomainEvent::PlatformDetected { os: "Linux".into() };
        assert!(e.validate_self());
    }
    #[test]
    fn event_name_roundtrip() {
        let e = DomainEvent::MigrationFailed {
            migration_id: "m1".into(),
            reason: "x".into(),
        };
        let s = serde_json::to_string(&e).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["type"], "MigrationFailed");
    }
}
